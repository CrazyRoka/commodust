# BASIC Interpreter Internals

This page documents the internal structure of the BASIC V2.0
interpreter, including routine entry points, variable storage
formats, and expression evaluation. This information is essential
for debugging BASIC programs and for understanding Kernal/BASIC
interaction.

## Floating Point Format

BASIC uses a 5-byte floating point format:

| Byte | Name | Description |
|---|---|---|
| 0 | Exponent | 128 = 2^0; 129 = 2^1; 0 = value is zero |
| 1–4 | Mantissa | 32-bit normalized mantissa (bit 7 of byte 1 = sign) |

The value = `mantissa × 2^(exponent - 128)`, where the mantissa
represents a number between 1.0 and 1.9999...

### FAC1 (Floating Point Accumulator #1)

| Address | Name | Description |
|---|---|---|
| $0061 | FACEXP | Exponent |
| $0062–$0065 | FACHO | 4-byte mantissa |
| $0066 | FACSGN | Sign byte (0 = positive, $FF = negative) |

### FAC2 (Floating Point Accumulator #2)

| Address | Name | Description |
|---|---|---|
| $0069 | ARGEXP | Exponent |
| $006A–$006D | ARGHO | 4-byte mantissa |
| $006E | ARGSGN | Sign byte |

## Variable Storage Format

### Non-Array Variables (7 bytes each)

| Bytes | Content |
|---|---|
| 0–1 | Variable name (2 ASCII chars; bit 7 of each byte encodes type) |
| 2–6 | Value (format depends on type) |

**Name encoding:**

| Byte 0 bit 7 | Byte 1 bit 7 | Type |
|---|---|---|
| 0 | 0 | Floating point |
| 1 | 0 | String |
| 0 | 1 | Integer |
| 1 | 1 | Function (FN) |

**Value formats:**

| Type | Bytes 2–6 |
|---|---|
| Float | 5-byte floating point (exponent + mantissa) |
| Integer | 2-byte value (high, low) + 3 unused |
| String | 1 byte length + 2 byte pointer to text + 2 unused |
| Function | 2 byte pointer to DEF text + 2 byte pointer to variable + 1 unused |

### Array Variables

| Bytes | Content |
|---|---|
| 0–1 | Array name (same encoding as non-array) |
| 2–3 | Offset to next array (low, high) |
| 4 | Number of dimensions |
| 5+ | Dimension sizes (2 bytes each, each = size + 1) |
| ... | Element values (packed: float=5, integer=2, string=3 bytes each) |

## BASIC Program Line Format

```
┌──────────────────────────────────────────────────────┐
│ Byte 0-1: Pointer to next line (low, high)            │
│           Two zeros ($00 $00) = end of program        │
│ Byte 2-3: Line number (low, high)                     │
│ Byte 4+:  Tokenized BASIC code                        │
│ Last byte: $00 (line terminator)                      │
└──────────────────────────────────────────────────────┘
```

Keywords are stored as single-byte tokens (values ≥ 128 / $80).
Text inside quotes is stored as literal ASCII. The first byte of
the BASIC text area must be $00 (end-of-program marker for NEW).

## CHRGET Routine ($0073–$008A)

A 24-byte self-modifying machine language subroutine copied from
ROM ($E3A2) to zero page at cold start. It is critical to BASIC
performance.

**Entry points:**

| Address | Label | Function |
|---|---|---|
| $0073 | CHRGET | Increment TXTPTR, fetch next character |
| $0079 | CHRGOT | Fetch current character (no increment) |
| $007A–$007B | TXTPTR | 2-byte pointer (self-modified LDA operand) |

**Exit status:**
- Accumulator: character read
- Carry: clear if ASCII digit 0–9, set otherwise
- Zero: set if character is $00 (end of line) or $3A (colon)

Because CHRGET is in RAM and self-modifying, it is the traditional
insertion point for "wedges" — code that intercepts BASIC to add
new commands.

## BASIC RAM Vector Table ($0300–$030B)

| Address | Label | Default | Function |
|---|---|---|---|
| $0300–$0301 | IERROR | $E38B | Error message handler |
| $0302–$0303 | IMAIN | $A483 | Main BASIC loop |
| $0304–$0305 | ICRNCH | $A57C | Tokenize ASCII to tokens |
| $0306–$0307 | IQPLOP | $A71A | List tokens as ASCII |
| $0308–$0309 | IGONE | $A7E4 | Execute next token |
| $030A–$030B | IEVAL | $AE86 | Evaluate arithmetic expression |

These vectors allow user code to intercept BASIC routines. Each
points one byte past a JMP instruction in ROM that indirectly
jumps through the corresponding RAM vector.

## Stack Usage by BASIC

### FOR/NEXT (18 bytes per active loop)

When a `FOR` loop is initialized, 18 bytes are pushed onto the 6510 stack (Page 1) in the following order:

| Offset | Size | Content |
|---|---|---|
| 0 | 1 | Constant `$81` (identifies this as a `FOR` block) |
| 1 | 2 | 16-bit Pointer to the loop index variable address (low, high) |
| 3 | 5 | 5-byte Floating Point representation of the `STEP` value |
| 8 | 1 | Sign of the `STEP` value (0 = positive, $FF = negative) |
| 9 | 5 | 5-byte Floating Point representation of the `TO` limit value |
| 14 | 2 | Line number to return to after `NEXT` (low, high) |
| 16 | 2 | Memory address of the next character to read in program text (low, high) |

*Note:* Because the `TO` expression is evaluated only once during loop initialization and its result is stored directly in this stack frame, modifying the limit variable inside the loop will not affect loop execution length. However, modifying the index variable directly affects the comparison test performed during `NEXT`.

### GOSUB (5 bytes per call)

When a `GOSUB` is executed, 5 bytes are pushed onto the stack:

| Offset | Size | Content |
|---|---|---|
| 0 | 1 | Constant `$8D` (identifies this as a `GOSUB` block) |
| 1 | 2 | 16-bit Line number to `RETURN` to (low, high) |
| 3 | 2 | 16-bit Pointer to the program text address where execution resumes after `RETURN` (low, high) |

*Note:* The `DEF FN` function utilizes an identical 5-byte stack frame structure, except that the identifier byte (Offset 0) contains a dummy placeholder value instead of `$8D`.

### DEF FN (5 bytes per definition)

Same as GOSUB, except byte 0 is a dummy value (not $8D).

## Garbage Collection

When string storage fills up, BASIC performs garbage collection:
1. Scans all string descriptors to find valid text in string storage.
2. Moves valid strings upward (toward higher memory).
3. Updates descriptors to new locations.
4. Adjusts FRETOP pointer ($33–$34).

This process can take several seconds and does not check the STOP
key. The duration is roughly the same whether any space is
recovered or not.

## Emulation Notes

If running real BASIC ROM, all of the above is handled by ROM
code. The emulator only needs to provide correct RAM and ROM
images. The CHRGET self-modification requires that zero page
be writable (which it always is — it's RAM).

The BASIC and Kernal ROMs are not contiguous ($A000–$BFFF and
$E000–$FFFF). The BASIC ROM ends with a `JMP $E000` instruction
to transfer control to Kernal routines. This means BASIC routine
addresses in the Kernal ROM are 3 bytes higher than their VIC-20
equivalents.