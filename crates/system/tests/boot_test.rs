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
