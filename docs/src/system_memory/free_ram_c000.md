# Free RAM at $C000–$CFFF

This 4 KB block of RAM ($C000–$CFFF) is the largest contiguous free
region in the C64's default memory map. It is not contiguous with
the BASIC program text area ($0800–$9FFF), so BASIC does not use it
for program or variable storage. It is not counted in the FRE(0)
total.

## Uses

### Machine Language Programs

The most common use for $C000–$CFFF is storing machine language
subroutines called via SYS from BASIC. The Kernal's SYS command
loads registers from $030C–$030F, calls the target address, and
stores registers back on return.

### Character Data

User-defined character sets can be placed here, particularly when
the VIC-II is banked to use the top 16K (bank 3, $C000–$FFFF). In
this bank, the Character ROM is not visible to the VIC-II, so all
character data must be supplied in RAM.

### Sprite Data

Up to 64 sprite shape blocks (64 bytes each = 4096 bytes) can fit
in this region. When using VIC-II bank 3, this is an ideal location
for sprite data.

### I/O Drivers

Custom I/O drivers (e.g., for parallel printers or IEEE-488 devices)
can be loaded here and hooked into the Kernal via the RAM I/O
vectors at $031A–$0333.

## Conflicts and Caveats

### DOS Support Wedge

The "Universal DOS Support" program (a common utility for easier
disk drive command channel access) typically loads at $CC00. Any
program using that area will overwrite the wedge, potentially
causing malfunctions.

### Simon's BASIC Cartridge

The Simon's BASIC extension cartridge uses several locations in
this range for its additional commands and routines.

### Bank Switching Considerations

When the Kernal ROM is switched out (HIRAM=0), this region remains
RAM — it is not affected by the LORAM/HIRAM/CHAREN banking signals.
It is always RAM to the CPU.

## Emulation Notes

This region requires no special handling. It is always backed by
physical RAM and is not affected by banking, mirroring, or I/O
overlays. The emulator can simply provide a flat 4K RAM block at
these addresses.

However, if the emulator supports loading binary files (e.g., `.prg`
files with a load address of $C000), it must correctly route the
data to this RAM region regardless of the current banking state.