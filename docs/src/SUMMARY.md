# Summary

- [Introduction](./introduction.md)

# Architecture
- [System Overview](./architecture/overview.md)
- [Memory Map](./architecture/memory_map.md)
- [Memory Banking and Configuration](./architecture/banking.md)

# CPU — MOS 6510
- [6510 Overview](./cpu/6510_overview.md)
- [On-Chip I/O Port (Locations $00–$01)](./cpu/io_port.md)
- [Zero Page ($0000–$00FF)](./cpu/zero_page.md)
- [Stack Page ($0100–$01FF)](./cpu/stack.md)
- [Interrupts and Vectors](./cpu/interrupts_vectors.md)

# System Memory
- [Pages 2 and 3 ($0200–$03FF)](./system_memory/pages_2_3.md)
- [Screen Memory and BASIC Text ($0400–$9FFF)](./system_memory/screen_and_basic_text.md)
- [Screen Editor Internals](./system_memory/screen_editor.md)
- [BASIC ROM ($A000–$BFFF)](./system_memory/basic_rom.md)
- [BASIC Interpreter Internals](./system_memory/basic_internals.md)
- [Kernal ROM ($E000–$FFFF)](./system_memory/kernal_rom.md)
- [Free RAM at $C000–$CFFF](./system_memory/free_ram_c000.md)

# VIC-II Video Chip
- [VIC-II Overview](./vic2/overview.md)
- [Pinout and Signals](./vic2/pinout.md)
- [Register Reference](./vic2/registers.md)
- [VIC-II Memory Addressing and Banking](./vic2/memory_addressing.md)
- [Internal Registers and Counters](./vic2/internal_registers.md)
- [Raster Line Timing and Memory Access](./vic2/timing_and_accesses.md)
- [Bad Lines and Bus Arbitration](./vic2/bad_lines.md)
- [Display Generation and Border Unit](./vic2/border_unit.md)
- [Text Modes](./vic2/text_modes.md)
- [Bitmap Modes](./vic2/bitmap_modes.md)
- [Invalid Graphics Modes](./vic2/invalid_modes.md)
- [Sprite Graphics](./vic2/sprites.md)
- [Raster Interrupts](./vic2/raster_interrupts.md)
- [Light Pen](./vic2/lightpen.md)
- [DRAM Refresh](./vic2/dram_refresh.md)
- [Known Effects and Quirks](./vic2/effects.md)

# SID Sound Chip
- [SID Overview](./sid/overview.md)
- [Voice Registers](./sid/voice_registers.md)
- [ADSR Envelopes](./sid/envelopes.md)
- [Filters](./sid/filters.md)
- [Miscellaneous Registers](./sid/miscellaneous.md)

# CIA Chips
- [CIA Overview](./cia/overview.md)
- [CIA #1 — Keyboard and Joystick](./cia/cia1.md)
- [CIA #2 — Serial Bus, RS-232, VIC Bank](./cia/cia2.md)
- [Timers](./cia/timers.md)
- [Time of Day Clock](./cia/tod_clock.md)

# Other Hardware
- [Color RAM ($D800–$DBFF)](./color_ram.md)
- [Character Generator ROM ($D000–$DFFF alt.)](./character_rom.md)

# I/O Devices
- [Keyboard](./io_devices/keyboard.md)
- [Joystick](./io_devices/joystick.md)
- [Game Paddles](./io_devices/paddles.md)
- [Cassette (Datassette)](./io_devices/cassette.md)
- [RS-232](./io_devices/rs232.md)
- [User Port](./io_devices/user_port.md)

# Appendices
- [Screen Color Codes](./appendices/color_codes.md)
- [Screen Codes](./appendices/screen_codes.md)
- [PETASCII Table](./appendices/petascii.md)
- [Keyboard Keycodes](./appendices/keycodes.md)