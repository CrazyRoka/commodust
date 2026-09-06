use system::{Prg, System};
use vic::VideoStandard;

#[test]
fn load_and_autorun_tokenized_basic_program() {
    let mut system = System::new(VideoStandard::Pal);

    // 1. Wait for system to reach READY prompt (~2.5s)
    for _ in 0..2_500_000 {
        system.tick();
    }

    // 2. Binary representation of: 10 PRINT "OK"
    //
    // Line layout:
    //   - Next line pointer: $080C (0x0C, 0x08)
    //   - Line number 10:    (0x0A, 0x00)
    //   - Token for PRINT:   $99
    //   - Literal "OK":      0x22, 0x4F, 0x4B, 0x22 ('"', 'O', 'K', '"')
    //   - End of line:       0x00
    //   - End of program:    0x00, 0x00
    let prg_bytes = [
        0x01, 0x08, // Load address: $0801
        0x0C, 0x08, // Line link pointer -> $080C
        0x0A, 0x00, // Line number 10
        0x99, // Token: PRINT
        0x22, 0x4F, 0x4B, 0x22, // "OK"
        0x00, // End of line
        0x00, 0x00, // End of program
    ];

    let prg = Prg::from_bytes(&prg_bytes).expect("PRG should be correctly loaded");

    // 3. Inject program and trigger autorun
    system.load_prg(&prg, true);

    // 4. Run for 200,000 cycles (~10 frames) to let BASIC parse RUN and execute
    for _ in 0..200_000 {
        system.tick();
    }

    // 5. Verify: "OK" must be printed on the screen!
    // Screen code for 'O' is 15 (0x0F), 'K' is 11 (0x0B)
    let ram = system.bus.ram();

    let mut found_ok = false;
    // Scan screen memory ($0400..$07E7) for the adjacent characters 'O' and 'K'
    for i in 0x0400..0x07E6 {
        if ram[i] == 15 && ram[i + 1] == 11 {
            found_ok = true;
            break;
        }
    }

    assert!(
        found_ok,
        "The text 'OK' must be printed on screen after running the PRG!"
    );
    println!("Successfully injected, auto-ran, and executed a .PRG program!");
}
