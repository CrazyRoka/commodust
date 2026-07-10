# Screen Color Codes

The C64 uses 4-bit color values (0–15) throughout its graphics systems.
All color registers and Color RAM use this encoding.

## Color Value Table

| Value | Hex | Color | Notes |
|---|---|---|---|
| 0 | $0 | Black | |
| 1 | $1 | White | |
| 2 | $2 | Red | |
| 3 | $3 | Cyan | Light blue-green |
| 4 | $4 | Purple | |
| 5 | $5 | Green | |
| 6 | $6 | Blue | Default background |
| 7 | $7 | Yellow | |
| 8 | $8 | Orange | |
| 9 | $9 | Brown | |
| 10 | $A | Light Red | |
| 11 | $B | Dark Gray | |
| 12 | $C | Medium Gray | Default Sprite 7 color |
| 13 | $D | Light Green | |
| 14 | $E | Light Blue | Default border color |
| 15 | $F | Light Gray | |

## PETASCII Color Control Characters

These PETASCII characters change the foreground color when printed to the
screen:

| Color | PETASCII | CHR$ | Key Combination |
|---|---|---|---|
| Black | 144 | CHR$(144) | CTRL-1 |
| White | 5 | CHR$(5) | CTRL-2 |
| Red | 28 | CHR$(28) | CTRL-3 |
| Cyan | 159 | CHR$(159) | CTRL-4 |
| Purple | 156 | CHR$(156) | CTRL-5 |
| Green | 30 | CHR$(30) | CTRL-6 |
| Blue | 31 | CHR$(31) | CTRL-7 |
| Yellow | 158 | CHR$(158) | CTRL-8 |
| Orange | 129 | CHR$(129) | C=-1 |
| Brown | 149 | CHR$(149) | C=-2 |
| Light Red | 150 | CHR$(150) | C=-3 |
| Dark Gray | 151 | CHR$(151) | C=-4 |
| Medium Gray | 152 | CHR$(152) | C=-5 |
| Light Green | 153 | CHR$(153) | C=-6 |
| Light Blue | 154 | CHR$(154) | C=-7 |
| Light Gray | 155 | CHR$(155) | C=-8 |

## Color Usage by Display Mode

### Standard Text Mode

| Pixel State | Color Source |
|---|---|
| Foreground (bit = 1) | Color RAM nybble (lower 4 bits) |
| Background (bit = 0) | Background Color Register 0 ($D021) |
| Border | Border Color Register ($D020) |

### Multicolor Text Mode

| Bit Pair | Color Source |
|---|---|
| 00 | $D021 (Background 0) |
| 01 | $D022 (Background 1) |
| 10 | $D023 (Background 2) |
| 11 | Color RAM nybble (lower 3 bits; bit 3 = multicolor flag) |

### Extended Background Color Text Mode

| Screen Code Range | Background Color |
|---|---|
| 0–63 | $D021 (Background 0) |
| 64–127 | $D022 (Background 1) |
| 128–191 | $D023 (Background 2) |
| 192–255 | $D024 (Background 3) |

Foreground color is always from Color RAM nybble.

### Standard Bitmap Mode

| Pixel State | Color Source |
|---|---|
| Foreground (bit = 1) | Upper nybble of video matrix byte |
| Background (bit = 0) | Lower nybble of video matrix byte |

Color RAM is not used.

### Multicolor Bitmap Mode

| Bit Pair | Color Source |
|---|---|
| 00 | $D021 (Background 0) |
| 01 | Upper nybble of video matrix byte |
| 10 | Lower nybble of video matrix byte |
| 11 | Color RAM nybble |

### Sprite Hi-Res Mode

| Pixel State | Color Source |
|---|---|
| 1 (opaque) | Sprite Color Register ($D027–$D02E) |
| 0 (transparent) | Background shows through |

### Sprite Multicolor Mode

| Bit Pair | Color Source |
|---|---|
| 00 | Transparent (background shows through) |
| 01 | Sprite Multicolor Register 0 ($D025) |
| 10 | Sprite Color Register ($D027–$D02E) |
| 11 | Sprite Multicolor Register 1 ($D026) |

## 4-Bit Register Notes

All VIC-II color registers ($D020–$D02E) and Color RAM ($D800–$DBFF) have
only the lower 4 bits connected. The upper 4 bits:
- **Writes:** Ignored (no effect)
- **Reads:** Return random or undefined values (varies by machine)

Always mask color register reads with `AND $0F` to extract the valid color
value.

## Default VIC-II Color Register Values

| Register | Address | Default | Color |
|---|---|---|---|
| EXTCOL (Border) | $D020 | $0E | Light Blue |
| BGCOL0 | $D021 | $06 | Blue |
| BGCOL1 | $D022 | $01 | White |
| BGCOL2 | $D023 | $02 | Red |
| BGCOL3 | $D024 | $03 | Cyan |
| SPMC0 | $D025 | $04 | Purple |
| SPMC1 | $D026 | $00 | Black |
| SP0COL | $D027 | $01 | White |
| SP1COL | $D028 | $02 | Red |
| SP2COL | $D029 | $03 | Cyan |
| SP3COL | $D02A | $04 | Purple |
| SP4COL | $D02B | $05 | Green |
| SP5COL | $D02C | $06 | Blue |
| SP6COL | $D02D | $07 | Yellow |
| SP7COL | $D02E | $0C | Medium Gray |