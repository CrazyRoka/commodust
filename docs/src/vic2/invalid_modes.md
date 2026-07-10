# Invalid Graphics Modes

Setting the ECM, BMM, and MCM bits to certain combinations does not select an "official" graphics mode, but creates internal behavior that can be exploited. The screen displays black, but the graphics data sequencer internally generates valid data that can trigger sprite collisions.

## Invalid Text Mode (ECM/BMM/MCM = 1/0/1)

The generated graphics are similar to multicolor text mode, but the character set is limited to 64 characters (due to ECM). All pixels are black.

### C-Access Address
Same as standard text mode.
**Data:** 12 bits. D11 = MC flag, D7-D0 = Character Pointer (limited to 6 bits).

### G-Access Address
| Bit | 13 | 12 | 11 | 10 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Value | CB13 | CB12 | CB11 | 0 | 0 | D5 | D4 | D3 | D2 | D1 | D0 | RC2 | RC1 | RC0 |

## Invalid Bitmap Mode 1 (ECM/BMM/MCM = 1/1/0)

Structure is as in standard bitmap mode, but bits 9 and 10 of the g-access address are zero, causing the graphics to repeat in four sections. All pixels are black.

### G-Access Address
| Bit | 13 | 12 | 11 | 10 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Value | CB13 | VC9 | VC8 | 0 | 0 | VC5 | VC4 | VC3 | VC2 | VC1 | VC0 | RC2 | RC1 | RC0 |

## Invalid Bitmap Mode 2 (ECM/BMM/MCM = 1/1/1)

Structure is as in multicolor bitmap mode, with the same address zeroing as Invalid Bitmap Mode 1. All pixels are black.

### Emulation Notes
Although the output is black, the emulator must still perform the memory fetches and sequencer logic. By placing sprites behind the "foreground" graphics (which are black), software can scan the internal state of the invalid modes using sprite-to-foreground collision detection.