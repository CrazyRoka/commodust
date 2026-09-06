use system::System;

fn main() {
    let mut system = System::new(vic::VideoStandard::Pal);

    for _ in 0..2_500_000 {
        system.tick();
    }
}
