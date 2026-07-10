# Screen Editor Internals

The C64's screen editor is implemented in Kernal ROM. It manages
cursor position, line editing, scrolling, quote mode, insert mode,
and color handling. This page documents the internal state variables
and behaviors that an emulator running real Kernal ROM must support.

## Screen Editor State Variables (Zero Page)

| Address | Name | Default | Function |
|---|---|---|---|
| $00C7 | RVS | $00 | Reverse video flag ($12 = on, $00 = off; reset at each CR) |
| $00C8 | INDX | — | End of logical line pointer (column of last non-blank, 0–79) |
| $00C9–$00CA | LXSP | — | Cursor X,Y position at start of input |
| $00CC | BLNSW | — | Cursor blink enable (0 = flash, nonzero = disabled) |
| $00CD | BLNCT | 20 | Cursor blink countdown (decremented every jiffy) |
| $00CE | GDBLN | — | Screen code of character under cursor |
| $00CF | BLNON | — | Cursor blink state (0 = reversed, 1 = normal) |
| $00D0 | CRSW | — | Input source flag (0 = keyboard, 3 = screen) |
| $00D1–$00D2 | PNT | — | Pointer to current screen line address in RAM |
| $00D3 | PNTR | — | Cursor column within current logical line (0–79) |
| $00D4 | CITSW | $00 | Quote mode flag (nonzero = quote mode active) |
| $00D5 | LNMX | 39 | Maximum physical line length |
| $00D6 | TBLX | — | Current cursor physical line number (0–24) |
| $00D7 | — | — | Temporary storage for last printed character ASCII |
| $00D8 | INSRT | $00 | Insert mode flag (>0 = number of pending inserts) |
| $00D9–$00F2 | LDTB1 | — | Screen line link table (25 entries) |
| $00F3–$00F4 | USER | — | Pointer to current Color RAM location |

## Screen Line Link Table ($D9–$F2)

25 entries, one per screen row. Each byte has two functions:

| Bits | Function |
|---|---|
| 0–3 | Page offset for screen line address calculation |
| 7 | Line linkage flag: 1 = first/only physical line of a logical line; 0 = second half of a logical line |

The high byte of each screen line address is calculated by adding
the page displacement from this table to the value in HIBASE ($0288).

### Logical Lines

A logical line can be 1 or 2 physical lines (40 or 80 characters).
When a character is typed past column 39 and the next physical line
is available, the lines are linked. The link table tracks which
pairs of physical lines form logical lines.

## Quote Mode

Quote mode is toggled each time a quotation mark (`"`, PETASCII 34)
is typed. In quote mode:

- Cursor control characters and other nonprinting characters appear
  as printed equivalents (reverse-video symbols) instead of executing.
- The DELETE key still functions normally in quote mode.
- Quote mode is exited by printing a closing quote, RETURN, or
  SHIFT-RETURN.

## Insert Mode

Activated by pressing INST. The editor:
1. Shifts the line right, opening a space.
2. Allocates another physical line to the logical line if needed.
3. Updates LNMX ($D5) and the line link table.
4. Sets INSRT ($D8) to the number of pending insert spaces.

Insert mode behaves like quote mode (nonprinting characters appear
as printed equivalents), except DELETE also leaves a printed
equivalent.

## Cursor Blink

| Variable | Value | Meaning |
|---|---|---|
| BLNSW ($CC) | 0 | Cursor blinking enabled |
| BLNSW ($CC) | nonzero | Cursor blinking disabled (during program run or buffer has chars) |
| BLNCT ($CD) | 20 | Initial countdown value |
| BLNON ($CF) | 0 | Cursor currently showing reversed character |
| BLNON ($CF) | 1 | Cursor showing normal character |

The cursor blinks approximately 3 times per second (20 jiffies ÷ 60 Hz).
The character under the cursor is stored in GDBLN ($CE) so it can be
restored when the cursor moves or blinks.

## Screen Scrolling

When the cursor reaches the bottom of the screen, the display scrolls
up one line. The scroll routine:

1. Moves all screen lines up by one physical line.
2. Moves the corresponding Color RAM lines up.
3. Clears the bottom line to spaces.
4. Sets the bottom Color RAM line to the current background color.
5. Updates the line link table.

Holding CTRL during a scroll causes a brief pause.

## Color Handling

| Location | Name | Function |
|---|---|---|
| $0286 | COLOR | Current foreground color (written to Color RAM on each PRINT) |
| $0287 | GDCOL | Saved color of character under cursor |

When a character is PRINTed, the value from COLOR ($0286) is written
to the corresponding Color RAM location. When the cursor moves over
a character, its color is saved in GDCOL ($0287) and restored when
the cursor moves away.

## Screen Memory Relocation

When relocating screen memory, three things must be updated:
1. `$D018` bits 4–7 (VIC-II video matrix pointer)
2. `$DD00` bits 0–1 (VIC-II bank, if changing banks)
3. `$0288` (HIBASE — OS screen memory page pointer)

If `$0288` is not updated, the OS will continue writing characters to
the old screen location while the VIC-II displays the new location,
resulting in invisible text.

## Emulation Notes

If the emulator runs real Kernal ROM, all screen editor state is
managed by the ROM code. The emulator only needs to provide:
- Correct VIC-II register behavior
- Correct CIA #1 keyboard scanning
- Correct RAM at all screen editor state locations

If using HLE (high-level emulation), the emulator must implement
all of the above state variables and behaviors itself.