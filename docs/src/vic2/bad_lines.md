# Bad Lines and Bus Arbitration

A "Bad Line" occurs when the VIC-II needs to fetch 40 bytes of video matrix data (c-accesses) and must steal bus cycles from the 6510 CPU to do so.

## Bad Line Condition

A Bad Line Condition is defined precisely as follows:

At the negative edge of ø0 at the beginning of a cycle, a Bad Line Condition exists if:
1. `RASTER` is between $30 and $F7 (inclusive).
2. The lower 3 bits of `RASTER` equal `YSCROLL` (bits 0-2 of $D011).
3. The `DEN` bit (bit 4 of $D011) was set at any point during raster line $30.

**Note:** The Bad Line Condition can be triggered or suppressed dynamically by modifying `YSCROLL` mid-frame.

## Bus Take-Over Sequence

When the VIC-II needs extra cycles (for c-accesses or sprite s-accesses), it initiates a bus take-over:

1. **BA goes Low:** The VIC-II pulls the BA line low 3 cycles before it needs the bus. This is connected to the 6510's RDY pin.
2. **Write Completion:** The 6510 ignores RDY during write cycles. The 3-cycle delay ensures the 6510 can finish any sequence of consecutive writes (max 3).
3. **AEC stays Low:** After the 3 cycles, AEC remains low during the second clock phase (ø2 high). This tri-states the 6510's address bus.
4. **VIC Access:** The VIC-II performs its reads during the second clock phase.

```
       _ _ _ _ _ _ _ _ _ _ _ _ _    _ _ _ _ _ _ _ _ _ _ _ _ _
 ø2   _ _ _ _ _ _ _ _ _ _ _ _ _ ..._ _ _ _ _ _ _ _ _ _ _ _ _
      ______________                       __________________
 BA                 ____________...________
       _ _ _ _ _ _ _ _ _ _                  _ _ _ _ _ _ _ _ _
 AEC  _ _ _ _ _ _ _ _ _ _ ______..._________ _ _ _ _ _ _ _ _ _

 Chip VPVPVPVPVPVPVPVpVpVpVVVVVV...VVVVVVVVVPVPVPVPVPVPVPVP

           1       |  2  |       3        |       4
         Normal    |Take-| VIC has taken  |  VIC releases
      bus activity |over | over the bus   |    the bus
```

## Timing Constraints

- **C-accesses:** On a Bad Line, the VIC-II performs 40 c-accesses between cycles 15 and 54. This requires the bus take-over to begin in cycle 12.
- **S-accesses:** Each active sprite requires 3 s-accesses. The VIC-II steals these cycles in statically assigned slots.
