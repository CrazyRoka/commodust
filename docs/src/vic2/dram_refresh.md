# DRAM Refresh

The VIC-II is responsible for refreshing the system's dynamic RAM (DRAM). It performs 5 read accesses in every raster line for refresh purposes.

## Refresh Mechanism

An internal 8-bit counter (`REF`) generates 256 DRAM row addresses.

- The counter is reset to `$FF` at raster line 0.
- It decrements by 1 after each refresh access.
- In line 0, the VIC accesses `$3FFF`, `$3FFE`, `$3FFD`, `$3FFC`, `$3FFB`.
- In line 1, it accesses `$3FFA` down to `$3FF6`, and so on.

### Refresh Address Format

| Bit | 13 | 12 | 11 | 10 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Value | 1 | 1 | 1 | 1 | 1 | 1 | REF7 | REF6 | REF5 | REF4 | REF3 | REF2 | REF1 | REF0 |

These accesses occur during the first phase of the clock cycle (ø2 low) and do not interfere with CPU execution.