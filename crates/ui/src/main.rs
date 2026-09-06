use ::keyboard::C64Key;
use iced::{
    ContentFit, Element, Length, Subscription, Task as Command,
    keyboard::{
        self,
        key::{Code, Physical},
    },
    time,
    widget::{
        Space, button, column, container,
        image::{self as iced_image, FilterMethod},
        pick_list, row, text,
    },
};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use system::System;
use vic::VideoStandard;

#[derive(Debug, Clone, PartialEq, Eq)]
enum MachineModel {
    Pal,
    Ntsc,
}

impl Display for MachineModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pal => "C64 PAL (50Hz)",
            Self::Ntsc => "C64 NTSC (60Hz)",
        })
    }
}

const SIDEBAR_WIDTH: f32 = 200.0;

// Full raster dimensions
const PAL_WIDTH: usize = 504;
const PAL_HEIGHT: usize = 312;
const NTSC_WIDTH: usize = 520;
const NTSC_HEIGHT: usize = 263;

// -----------------------------------------------------------------------------
// The Canonical 16-Color C64 Palette (Pepto Palette) in RGBA8
// -----------------------------------------------------------------------------
const C64_PALETTE_RGBA: [[u8; 4]; 16] = [
    [0x00, 0x00, 0x00, 0xFF], // 0: Black
    [0xFF, 0xFF, 0xFF, 0xFF], // 1: White
    [0x88, 0x00, 0x00, 0xFF], // 2: Red
    [0xAA, 0xFF, 0xEE, 0xFF], // 3: Cyan
    [0xCC, 0x44, 0xCC, 0xFF], // 4: Purple
    [0x00, 0xCC, 0x55, 0xFF], // 5: Green
    [0x00, 0x00, 0xAA, 0xFF], // 6: Blue
    [0xEE, 0xEE, 0x77, 0xFF], // 7: Yellow
    [0xDD, 0x88, 0x55, 0xFF], // 8: Orange
    [0x66, 0x44, 0x00, 0xFF], // 9: Brown
    [0xFF, 0x77, 0x77, 0xFF], // 10: Light Red
    [0x33, 0x33, 0x33, 0xFF], // 11: Dark Gray
    [0x77, 0x77, 0x77, 0xFF], // 12: Medium Gray
    [0xAA, 0xFF, 0x66, 0xFF], // 13: Light Green
    [0x00, 0x88, 0xFF, 0xFF], // 14: Light Blue
    [0xBB, 0xBB, 0xBB, 0xFF], // 15: Light Gray
];

pub fn main() -> iced::Result {
    iced::application(EmulatorApp::new, EmulatorApp::update, EmulatorApp::view)
        .subscription(EmulatorApp::subscription)
        .window_size((
            PAL_WIDTH as f32 * 1.5 + SIDEBAR_WIDTH,
            PAL_HEIGHT as f32 * 2.5,
        ))
        .title(EmulatorApp::title)
        .run()
}

// -----------------------------------------------------------------------------
// Shared State between Emulator Thread and UI
// -----------------------------------------------------------------------------
struct SharedState {
    frame_buffer: Vec<u8>, // RGBA8 buffer for Iced Image
    current_fps: f32,
    width: u32,
    height: u32,
}

// Commands sent from UI -> Emulator Thread
enum EmuCommand {
    Restart,
    ToggleFpsLimit,
    SetModel(MachineModel),
    KeyDown(C64Key),
    KeyUp(C64Key),
}

// -----------------------------------------------------------------------------
// Iced UI Application
// -----------------------------------------------------------------------------
struct EmulatorApp {
    shared_state: Arc<Mutex<SharedState>>,
    command_tx: std::sync::mpsc::Sender<EmuCommand>,
    unlimited_fps: bool,
    filter_method: FilterMethod,
    selected_model: MachineModel,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ModelSelected(MachineModel),
    RestartPressed,
    ToggleFpsPressed,
    ToggleFilterMethod,
    EventProcessed(iced::Event),
}

impl EmulatorApp {
    fn new() -> (Self, Command<Message>) {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let shared_state = Arc::new(Mutex::new(SharedState {
            frame_buffer: vec![0; PAL_WIDTH * PAL_HEIGHT * 4],
            current_fps: 0.0,
            width: PAL_WIDTH as u32,
            height: PAL_HEIGHT as u32,
        }));

        // Spawn Emulator Thread
        std::thread::spawn({
            let state_clone = Arc::clone(&shared_state);
            move || run_emulator_thread(command_rx, state_clone)
        });

        (
            Self {
                shared_state,
                command_tx,
                unlimited_fps: false,
                filter_method: FilterMethod::Nearest,
                selected_model: MachineModel::Pal,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let state = self.shared_state.lock().unwrap();
        format!("Commodore 64 Emulator - FPS: {:.1}", state.current_fps)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ModelSelected(model) => {
                self.selected_model = model.clone();
                let _ = self.command_tx.send(EmuCommand::SetModel(model));
            }
            Message::RestartPressed => {
                let _ = self.command_tx.send(EmuCommand::Restart);
            }
            Message::ToggleFpsPressed => {
                self.unlimited_fps = !self.unlimited_fps;
                let _ = self.command_tx.send(EmuCommand::ToggleFpsLimit);
            }
            Message::ToggleFilterMethod => {
                self.filter_method = match self.filter_method {
                    FilterMethod::Nearest => FilterMethod::Linear,
                    FilterMethod::Linear => FilterMethod::Nearest,
                };
            }
            Message::EventProcessed(iced::Event::Keyboard(event)) => match event {
                keyboard::Event::KeyPressed { physical_key, .. } => {
                    if let Physical::Code(code) = physical_key {
                        if code == Code::F12 {
                            self.unlimited_fps = !self.unlimited_fps;
                            let _ = self.command_tx.send(EmuCommand::ToggleFpsLimit);
                        } else if let Some(code) = map_keycode(code) {
                            let _ = self.command_tx.send(EmuCommand::KeyDown(code));
                        }
                    }
                }
                keyboard::Event::KeyReleased { physical_key, .. } => {
                    if let Physical::Code(code) = physical_key {
                        if code != Code::F12 {
                            if let Some(code) = map_keycode(code) {
                                let _ = self.command_tx.send(EmuCommand::KeyUp(code));
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let (current_fps, frame_data, width, height) = {
            let state = self.shared_state.lock().unwrap();
            (
                state.current_fps,
                state.frame_buffer.clone(),
                state.width,
                state.height,
            )
        };

        // Sidebar Control Panel
        let sidebar = column![
            text(format!("{}", self.selected_model)).size(22),
            Space::new().height(Length::Fixed(20.0)),
            text("Standard:"),
            pick_list(
                vec![MachineModel::Pal, MachineModel::Ntsc],
                Some(self.selected_model.clone()),
                Message::ModelSelected
            )
            .width(Length::Fill),
            Space::new().height(Length::Fixed(20.0)),
            button(text("Restart"))
                .width(Length::Fill)
                .on_press(Message::RestartPressed),
            Space::new().height(Length::Fixed(10.0)),
            button(text(if self.unlimited_fps {
                "Lock FPS"
            } else {
                "Unlock FPS (Fast)"
            }))
            .width(Length::Fill)
            .on_press(Message::ToggleFpsPressed),
            Space::new().height(Length::Fixed(10.0)),
            button(text(match self.filter_method {
                FilterMethod::Nearest => "Filter: Nearest",
                FilterMethod::Linear => "Filter: Linear",
            }))
            .width(Length::Fill)
            .on_press(Message::ToggleFilterMethod),
            Space::new().height(Length::Fill),
            text(format!("FPS: {:.1}", current_fps)).size(14),
        ]
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .padding(15)
        .spacing(10);

        // Emulator Screen
        let handle = iced_image::Handle::from_rgba(width, height, frame_data);

        let screen = container(
            iced_image::Image::new(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .filter_method(self.filter_method),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Alignment::Center)
        .align_y(iced::alignment::Alignment::Center);

        row![sidebar, screen].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(Duration::from_millis(16)).map(|_| Message::Tick),
            iced::event::listen().map(Message::EventProcessed),
        ])
    }
}

// -----------------------------------------------------------------------------
// Core Emulator Background Thread
// -----------------------------------------------------------------------------
fn run_emulator_thread(
    rx: std::sync::mpsc::Receiver<EmuCommand>,
    shared_state: Arc<Mutex<SharedState>>,
) {
    let mut model = MachineModel::Pal;
    let mut system = System::new(VideoStandard::Pal);

    let mut unlimited_fps = false;
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();
    let mut last_frame_time = Instant::now();

    loop {
        // Handle UI Commands
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                EmuCommand::Restart => {
                    system = System::new(match model {
                        MachineModel::Pal => VideoStandard::Pal,
                        MachineModel::Ntsc => VideoStandard::Ntsc,
                    });
                }
                EmuCommand::ToggleFpsLimit => unlimited_fps = !unlimited_fps,
                EmuCommand::SetModel(m) => {
                    model = m;
                    system = System::new(match model {
                        MachineModel::Pal => VideoStandard::Pal,
                        MachineModel::Ntsc => VideoStandard::Ntsc,
                    });
                }
                EmuCommand::KeyDown(code) => {
                    system.press_key(&code);
                }
                EmuCommand::KeyUp(code) => {
                    system.release_key(&code);
                }
            }
        }

        // ---------------------------------------------------------------------
        // Run Exactly 1 Frame Worth of Cycles
        // ---------------------------------------------------------------------
        let cycles_per_frame = match model {
            MachineModel::Pal => PAL_HEIGHT * 63,   // 19,656 cycles
            MachineModel::Ntsc => NTSC_HEIGHT * 65, // 17,095 cycles
        };

        for _ in 0..cycles_per_frame {
            system.tick();
        }

        // ---------------------------------------------------------------------
        // Convert VIC-II Color Indices to RGBA8 Framebuffer
        // ---------------------------------------------------------------------
        {
            let (width, height) = match model {
                MachineModel::Pal => (PAL_WIDTH as u32, PAL_HEIGHT as u32),
                MachineModel::Ntsc => (NTSC_WIDTH as u32, NTSC_HEIGHT as u32),
            };

            let vic_buffer = system.bus.vic().framebuffer();
            let mut state = shared_state.lock().unwrap();

            state.width = width;
            state.height = height;

            // Re-allocate only if dimensions changed (e.g. PAL <-> NTSC)
            let required_len = vic_buffer.len() * 4;
            if state.frame_buffer.len() != required_len {
                state.frame_buffer = vec![0; required_len];
            }

            // Translate 4-bit C64 color index -> RGBA
            for (idx, &color_idx) in vic_buffer.iter().enumerate() {
                let rgba = C64_PALETTE_RGBA[(color_idx & 0x0F) as usize];
                let out_offset = idx * 4;
                state.frame_buffer[out_offset..out_offset + 4].copy_from_slice(&rgba);
            }
        }

        // FPS calculation
        frame_count += 1;
        let elapsed = last_fps_update.elapsed();
        if elapsed.as_secs_f32() >= 0.5 {
            let fps = frame_count as f32 / elapsed.as_secs_f32();
            shared_state.lock().unwrap().current_fps = fps;
            frame_count = 0;
            last_fps_update = Instant::now();
        }

        // Throttle to 50Hz (PAL: 20ms) or 60Hz (NTSC: 16.6ms)
        if !unlimited_fps {
            let target_frame_time = match model {
                MachineModel::Pal => Duration::from_millis(20),
                MachineModel::Ntsc => Duration::from_millis(16),
            };
            let time_taken = last_frame_time.elapsed();
            if time_taken < target_frame_time {
                std::thread::sleep(target_frame_time - time_taken);
            }
            last_frame_time = Instant::now();
        } else {
            last_frame_time = Instant::now();
        }
    }
}

fn map_keycode(code: Code) -> Option<C64Key> {
    match code {
        Code::Enter | Code::NumpadEnter => Some(C64Key::Return),
        Code::Space => Some(C64Key::Space),
        Code::Backspace => Some(C64Key::Del),
        Code::ShiftLeft => Some(C64Key::LeftShift),
        Code::ShiftRight => Some(C64Key::RightShift),
        Code::ControlLeft | Code::ControlRight => Some(C64Key::Ctrl),
        Code::Escape => Some(C64Key::RunStop),
        Code::Home => Some(C64Key::Home),

        // Letters
        Code::KeyA => Some(C64Key::A),
        Code::KeyB => Some(C64Key::B),
        Code::KeyC => Some(C64Key::C),
        Code::KeyD => Some(C64Key::D),
        Code::KeyE => Some(C64Key::E),
        Code::KeyF => Some(C64Key::F),
        Code::KeyG => Some(C64Key::G),
        Code::KeyH => Some(C64Key::H),
        Code::KeyI => Some(C64Key::I),
        Code::KeyJ => Some(C64Key::J),
        Code::KeyK => Some(C64Key::K),
        Code::KeyL => Some(C64Key::L),
        Code::KeyM => Some(C64Key::M),
        Code::KeyN => Some(C64Key::N),
        Code::KeyO => Some(C64Key::O),
        Code::KeyP => Some(C64Key::P),
        Code::KeyQ => Some(C64Key::Q),
        Code::KeyR => Some(C64Key::R),
        Code::KeyS => Some(C64Key::S),
        Code::KeyT => Some(C64Key::T),
        Code::KeyU => Some(C64Key::U),
        Code::KeyV => Some(C64Key::V),
        Code::KeyW => Some(C64Key::W),
        Code::KeyX => Some(C64Key::X),
        Code::KeyY => Some(C64Key::Y),
        Code::KeyZ => Some(C64Key::Z),

        // Digits
        Code::Digit0 => Some(C64Key::Zero),
        Code::Digit1 => Some(C64Key::One),
        Code::Digit2 => Some(C64Key::Two),
        Code::Digit3 => Some(C64Key::Three),
        Code::Digit4 => Some(C64Key::Four),
        Code::Digit5 => Some(C64Key::Five),
        Code::Digit6 => Some(C64Key::Six),
        Code::Digit7 => Some(C64Key::Seven),
        Code::Digit8 => Some(C64Key::Eight),
        Code::Digit9 => Some(C64Key::Nine),

        // Symbols
        Code::Equal => Some(C64Key::Equal),
        Code::Minus => Some(C64Key::Minus),
        Code::NumpadAdd => Some(C64Key::Plus),
        Code::Comma => Some(C64Key::Comma),
        Code::Period => Some(C64Key::Dot),
        Code::Slash => Some(C64Key::Slash),
        Code::Semicolon => Some(C64Key::Colon),
        Code::Quote => Some(C64Key::Semicolon),

        // Function Keys
        Code::F1 => Some(C64Key::F1),
        Code::F3 => Some(C64Key::F3),
        Code::F5 => Some(C64Key::F5),
        Code::F7 => Some(C64Key::F7),

        // Cursor
        Code::ArrowUp | Code::ArrowDown => Some(C64Key::CrsrVertical),
        Code::ArrowLeft | Code::ArrowRight => Some(C64Key::CrsrHorizontal),

        _ => None,
    }
}
