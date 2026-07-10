# Stack Page ($0100–$01FF)

The 6510 microprocessor dedicates page 1 of memory ($0100–$01FF) to
a hardware-implemented stack. The stack grows downward from $01FF
toward $0100. The Stack Pointer (SP) register is 8 bits wide; the
CPU implicitly ORs it with $01 to form the full 16-bit address.

## Stack Operations

| Operation | Instruction | Stack Effect |
|---|---|---|
| Push A | PHA | SP decrements; A → $01SP |
| Pull A | PLA | SP increments; $01SP → A |
| Push P | PHP | SP decrements; P → $01SP |
| Pull P | PLP | SP increments; $01SP → P |
| JSR | JSR addr | SP -= 2; return addr (PC-1) pushed (high, low) |
| Return | RTS | SP += 2; return addr pulled → PC |
| Interrupt | IRQ/NMI | SP -= 3; P, PC(high), PC(low) pushed |
| Return | RTI | SP += 3; PC and P pulled |
| BRK | BRK | Same as IRQ (B bit set in pushed P) |

### Push Order for Interrupts

When an IRQ or NMI occurs, the CPU pushes in this order:

1. PC High Byte → $01SP, SP--
2. PC Low Byte → $01SP, SP--
3. P (with B=0, bit5=1) → $01SP, SP--

RTS/RTI pull in reverse order.

## Memory Layout Within the Stack Page

### $0100–$010A: Floating Point to String Conversion Work Area

Used by the routines that convert numbers to ASCII digits and during
string scanning.

### $0100–$013E: BAD — Tape Input Error Log

62 bytes used as indices to track which bytes in a tape block were
received incorrectly during the first pass. Corrections are made on
the second pass (each tape block is written twice).

### $013F–$01FF: Actual Stack Space

The remaining 193 bytes are available for the 6510 hardware stack.
BASIC and the Kernal check for at least 62 bytes of free stack
before performing stack-heavy operations; an OUT OF MEMORY error is
issued if insufficient space exists.

## BASIC Stack Usage

### FOR/NEXT Loop Entries (18 bytes per active loop)

Each active FOR loop pushes 18 bytes onto the stack in this order
(first pushed listed first):

| Offset | Size | Content |
|---|---|---|
| 0 | 1 | Constant $81 (identifies FOR block) |
| 1 | 2 | Pointer to loop variable address |
| 3 | 5 | STEP value (floating point) |
| 8 | 1 | Sign of STEP |
| 9 | 5 | TO/limit value (floating point) |
| 14 | 2 | Line number to return to after NEXT |
| 16 | 2 | Address of next character to read after FOR |

### GOSUB Entries (5 bytes per call)

| Offset | Size | Content |
|---|---|---|
| 0 | 1 | Constant $8D (identifies GOSUB block) |
| 1 | 2 | Line number to RETURN to |
| 3 | 2 | Pointer to address of BASIC text at RETURN point |

### DEF FN Entries (5 bytes per definition)

Same layout as GOSUB, except byte 0 is a dummy value (not $8D).

## Stack Limits and Error Conditions

- **Stack overflow:** If more than 256 bytes are pushed, SP wraps
  from $00 to $FF, overwriting previous entries. This usually crashes
  the system.
- **Stack underflow:** If more items are pulled than pushed, SP wraps
  from $FF to $00. Also typically fatal.
- **BASIC check:** Before FOR, GOSUB, and expression evaluation,
  BASIC checks if at least 62 bytes of stack remain. If not, OUT OF
  MEMORY error is issued.

## Emulation Notes

The stack is simply part of the 64K RAM array at $0100–$01FF. The
emulator need not allocate separate storage. The SP register is
8-bit and the CPU prepends $01 to form the address.

For save-state functionality, the SP register value must be stored
along with the full contents of $0100–$01FF, since data below SP
may still be relevant if the program reads the stack area directly
(some programs do this for debugging or custom interrupt handling).