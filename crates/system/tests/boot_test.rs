use keyboard::C64Key;
use system::System;

#[test]
fn pal_system_boots_to_basic_prompt() {
    let mut system = System::new(vic::VideoStandard::Pal);

    // Run for 2,500,000 cycles (~2.5 seconds of emulated time).
    // This is enough time for the Kernal to finish RAMTAS, IOINIT, CINT,
    // print the startup banner, and yield the "READY." prompt.
    for _ in 0..2_500_000 {
        system.tick();
    }

    // 1. Verify PAL detection succeeded ($02A6 should be 1)
    let ram = system.bus.ram();
    assert_eq!(ram[0x02A6], 1, "Kernal must auto-detect PAL ($02A6 == 1)");

    // 2. Row 0 ($0400..$0427) is a blank line (all spaces = 32)
    assert_eq!(ram[0x0400], 0x20, "Row 0 should be blank spaces");

    // 3. Row 1 ($0428..$044F) contains "    **** COMMODORE 64 BASIC V2 ****"
    // The 4 asterisks start at offset 4 of Row 1: $0428 + 4 = $042C
    assert_eq!(ram[0x042C], 0x2A, "Expected '*' at $042C");
    assert_eq!(ram[0x042D], 0x2A, "Expected '*' at $042D");
    assert_eq!(ram[0x042E], 0x2A, "Expected '*' at $042E");
    assert_eq!(ram[0x042F], 0x2A, "Expected '*' at $042F");

    // Print the first 6 screen rows so you can see your booted C64!
    println!("\n--- C64 SCREEN MEMORY DUMP ---");
    for row in 0..6 {
        let start = 0x0400 + (row * 40);
        let line = &ram[start..start + 40];
        print!("Row {row:02}: |");
        for &code in line {
            let ch = match code {
                0x20 => ' ',
                0x2A => '*',
                0x01..=0x1A => (code + 0x40) as char,
                0x30..=0x39 => code as char,
                _ => '.',
            };
            print!("{}", ch);
        }
        println!("|");
    }
    println!("------------------------------\n");
}

#[test]
fn typing_a_character_appears_at_cursor_in_basic() {
    let mut system = System::new(vic::VideoStandard::Pal);

    // 1. Boot to BASIC prompt (~2.5 seconds)
    for _ in 0..2_500_000 {
        system.tick();
    }

    // Row 5 has "READY." (columns 0..=5). The cursor sits at column 0 of Row 6!
    assert_eq!(system.bus.ram()[0x0400 + (5 * 40)], 18); // Screen code 18 = 'R'

    // 2. Press key 'A'
    system.press_key(&C64Key::A);

    // Run for 100,000 cycles (~5 frames / 0.1s) with 'A' held down
    // The Kernal 50Hz IRQ scans the matrix, debounces, buffers PETASCII 'A',
    // and direct-mode BASIC prints screen code 1 ('A') at the cursor!
    for _ in 0..100_000 {
        system.tick();
    }

    // 3. Release key 'A'
    system.release_key(&C64Key::A);

    // Run another 50,000 cycles to allow cursor to advance
    for _ in 0..50_000 {
        system.tick();
    }

    // 4. Verify that Screen Code 1 ('A') appeared on Row 6!
    let row6_col0 = system.bus.ram()[0x0400 + (6 * 40)];
    assert_eq!(
        row6_col0, 1,
        "Expected Screen Code 1 ('A') on Row 6 after typing 'A', got {}",
        row6_col0
    );

    // println!("Successfully typed 'A' into Commodore 64 BASIC!");
}

#[test]
fn typing_multiple_characters_and_shift_work() {
    let mut system = System::new(vic::VideoStandard::Pal);
    for _ in 0..2_500_000 {
        system.tick();
    }

    // Press/release 'A': ~3 frames is enough for scan + debounce + echo
    system.press_key(&C64Key::A);
    for _ in 0..60_000 {
        system.tick();
    }
    system.release_key(&C64Key::A);
    for _ in 0..60_000 {
        system.tick();
    }

    // SHIFT + 8 = '(' — proves the modifier path (two keys, different
    // row AND column, detected simultaneously by the 2-key Kernal scan)
    system.press_key(&C64Key::LeftShift);
    system.press_key(&C64Key::Eight);
    for _ in 0..60_000 {
        system.tick();
    }
    system.release_key(&C64Key::Eight);
    system.release_key(&C64Key::LeftShift);
    for _ in 0..60_000 {
        system.tick();
    }

    let ram = system.bus.ram();
    assert_eq!(
        ram[0x0400 + 6 * 40 + 0],
        0x01,
        "expected 'A' (screen code 1)"
    );
    assert_eq!(
        ram[0x0400 + 6 * 40 + 1],
        0x28,
        "expected '(' (screen code 0x28)"
    );
}
