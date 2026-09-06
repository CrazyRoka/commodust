use std::collections::HashSet;

use crate::C64Key;

pub struct Keyboard {
    pressed_keys: HashSet<C64Key>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: Default::default(),
        }
    }

    pub fn press_key(&mut self, key: &C64Key) {
        self.pressed_keys.insert(key.clone());
    }

    pub fn release_key(&mut self, key: &C64Key) {
        self.pressed_keys.remove(key);
    }

    pub fn reset(&mut self) {
        self.pressed_keys.clear();
    }

    pub fn is_pressed(&self, key: &C64Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn scan_rows(&self, columns: u8) -> u8 {
        let mut result = 0xFF;
        for column in 0..8 {
            if (1 << column) & columns != 0 {
                continue;
            }

            for (row, keys) in C64Key::ALL_ROWS.iter().enumerate() {
                if self.is_pressed(&keys[column]) {
                    result &= !(1 << row);
                }
            }
        }

        result
    }

    pub fn scan_columns(&self, rows: u8) -> u8 {
        let mut result = 0xFF;
        for row in 0..8 {
            if (1 << row) & rows != 0 {
                continue;
            }

            for (column, key) in C64Key::ALL_ROWS[row].iter().enumerate() {
                if self.is_pressed(key) {
                    result &= !(1 << column);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================================================================
    // CONTRACT — the behaviour pinned down by these tests.
    //
    // 1. INITIAL / RESET STATE
    //    * No keys pressed.
    //    * scan_rows() returns 0xFF for any column mask (all pull-ups high).
    //    * scan_columns() returns 0xFF for any row mask.
    //    * is_pressed(key) is false for every key.
    //
    // 2. ACTIVE-LOW MATRIX SCANNING
    //    * Columns (Port A) are active-low: bit C = 0 selects Column C.
    //    * Rows (Port B) are active-low: bit R = 0 means Row R is grounded.
    //    * If key at (Row R, Col C) is pressed:
    //      - When Column C is selected (bit C = 0), bit R of scan_rows is 0.
    //      - When Column C is NOT selected (bit C = 1), bit R of scan_rows is 1.
    //
    // 3. ALL 64 MATRIX KEYS UNIQUENESS
    //    * Every one of the 64 keys in ALL_KEYS maps to an exact, unique
    //      (row, col) coordinate.
    //
    // 4. MULTI-KEY ROLLOVER (N-Key Rollover & Modifiers)
    //    * Multiple keys pressed simultaneously on different rows and columns
    //      ground their respective lines independently without cross-talk.
    //    * Simultaneous modifier keys (e.g. LeftShift + A) work cleanly.
    //
    // 5. GLOBAL MATRIX POLL ($DC00 = 0x00)
    //    * Kernal uses Port A = 0x00 to test if ANY key is pressed.
    //    * Any pressed key grounds its corresponding row bit when 0x00 is passed.
    //
    // 6. IDEMPOTENCE & RELEASE
    //    * Pressing an already-pressed key is a no-op.
    //    * Releasing an unpressed key is a no-op.
    //    * Releasing a key restores row levels to 0xFF.
    //    * reset() clears all pressed keys.
    // =====================================================================

    fn key_at(row: usize, col: usize) -> C64Key {
        C64Key::ALL_ROWS[row][col]
    }

    fn coord_of(key: C64Key) -> (usize, usize) {
        for (row, keys) in C64Key::ALL_ROWS.iter().enumerate() {
            for (col, &k) in keys.iter().enumerate() {
                if k == key {
                    return (row, col);
                }
            }
        }
        panic!("key {key:?} not present in the 8x8 matrix");
    }

    fn press_all(kbd: &mut Keyboard, keys: &[C64Key]) {
        for &key in keys {
            kbd.press_key(&key);
        }
    }

    /// Reference (oracle) model: row R is low iff some pressed key sits at
    /// (R, C) with column line C driven low in `columns`.
    fn reference_rows(pressed: &[C64Key], columns: u8) -> u8 {
        let mut rows = 0xFF;
        for &key in pressed {
            let (row, col) = coord_of(key);
            if columns & (1 << col) == 0 {
                rows &= !(1 << row);
            }
        }
        rows
    }

    fn reference_columns(pressed: &[C64Key], rows: u8) -> u8 {
        let mut columns = 0xFF;
        for &key in pressed {
            let (row, col) = coord_of(key);
            if rows & (1 << row) == 0 {
                columns &= !(1 << col);
            }
        }
        columns
    }

    /// Reproduces the KERNAL SCNKEY walk: drive each column low in turn
    /// ($DC00 = $FE, $FD, ..., $7F) and collect every (row, col) that
    /// reads back low from $DC01.
    fn kernal_column_walk(kbd: &Keyboard) -> Vec<C64Key> {
        let mut found = Vec::new();
        for col in 0..8 {
            let rows = kbd.scan_rows(!(1 << col));
            for row in 0..8 {
                if rows & (1 << row) == 0 {
                    found.push(key_at(row, col));
                }
            }
        }
        found
    }

    /// Mirror-image walk some software uses: drive rows from $DC01 and
    /// read the column state back from $DC00.
    fn reverse_row_walk(kbd: &Keyboard) -> Vec<C64Key> {
        let mut found = Vec::new();
        for row in 0..8 {
            let columns = kbd.scan_columns(!(1 << row));
            for col in 0..8 {
                if columns & (1 << col) == 0 {
                    found.push(key_at(row, col));
                }
            }
        }
        found
    }

    /// Tiny deterministic PRNG (no external deps) so fuzz tests reproduce.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 16
        }

        fn below(&mut self, n: u64) -> u64 {
            assert!(n > 0);
            self.next() % n
        }
    }

    // ---------------------------------------------------------------------
    // 1. Initial State
    // ---------------------------------------------------------------------

    #[test]
    fn new_keyboard_has_no_keys_pressed_and_lines_float_high() {
        let kbd = Keyboard::new();

        // No keys pressed: any scan of columns returns 0xFF (pull-ups high)
        assert_eq!(kbd.scan_rows(0x00), 0xFF);
        assert_eq!(kbd.scan_rows(0xFE), 0xFF);
        assert_eq!(kbd.scan_rows(0xFF), 0xFF);
        assert_eq!(kbd.scan_columns(0x00), 0xFF);
        assert_eq!(kbd.scan_columns(0xFE), 0xFF);
        assert_eq!(kbd.scan_columns(0xFF), 0xFF);

        for &key in &C64Key::ALL_KEYS {
            assert!(!kbd.is_pressed(&key));
        }
    }

    // ---------------------------------------------------------------------
    // 2. Single Key Press & Active-Low Column Selection
    // ---------------------------------------------------------------------

    #[test]
    fn pressing_space_grounds_row_7_when_column_4_is_selected() {
        // Space is at Row 7, Column 4
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::Space);

        assert!(kbd.is_pressed(&C64Key::Space));

        // Column 4 selected: Bit 4 of Port A is 0 -> 0b1110_1111 (0xEF)
        // Row 7 should drop to 0 -> 0b0111_1111 (0x7F)
        assert_eq!(
            kbd.scan_rows(0xEF),
            0x7F,
            "Row 7 must be low when Column 4 is selected"
        );

        // Any other column selected: Space must NOT register!
        // E.g. Column 0 selected (0xFE) -> 0xFF
        assert_eq!(
            kbd.scan_rows(0xFE),
            0xFF,
            "Space must not ground Row 7 when Column 0 is selected"
        );

        // When NO columns are selected (0xFF) -> 0xFF
        assert_eq!(kbd.scan_rows(0xFF), 0xFF);
    }

    #[test]
    fn releasing_space_restores_high_lines() {
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::Space);
        assert_eq!(kbd.scan_rows(0xEF), 0x7F);

        kbd.release_key(&C64Key::Space);
        assert!(!kbd.is_pressed(&C64Key::Space));
        assert_eq!(kbd.scan_rows(0xEF), 0xFF);
    }

    // ---------------------------------------------------------------------
    // 3. Exhaustive Verification of All 64 Keys
    // ---------------------------------------------------------------------

    #[test]
    fn all_64_keys_ground_their_exact_row_and_column() {
        let matrix = [
            C64Key::ROW0,
            C64Key::ROW1,
            C64Key::ROW2,
            C64Key::ROW3,
            C64Key::ROW4,
            C64Key::ROW5,
            C64Key::ROW6,
            C64Key::ROW7,
        ];

        for (row, keys) in matrix.iter().enumerate() {
            for (col, &key) in keys.iter().enumerate() {
                let mut kbd = Keyboard::new();
                kbd.press_key(&key);

                // Port A selects Column `col`: bit `col` is 0
                let col_mask = !(1 << col);
                // Port B should show Row `row` low: bit `row` is 0
                let expected_row_mask = !(1 << row);

                assert_eq!(
                    kbd.scan_rows(col_mask),
                    expected_row_mask,
                    "Key {:?} at Row {}, Col {} did not ground correctly",
                    key,
                    row,
                    col
                );

                // When other columns are selected, this key must not ground
                for other_col in 0..8 {
                    if other_col != col {
                        assert_eq!(
                            kbd.scan_rows(!(1 << other_col)),
                            0xFF,
                            "Key {:?} leaked into Column {}",
                            key,
                            other_col
                        );
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // 4. Global Matrix Polling (Port A = 0x00)
    // ---------------------------------------------------------------------

    #[test]
    fn global_poll_grounds_row_for_any_pressed_key() {
        let mut kbd = Keyboard::new();

        // Key 'A' is Row 1, Column 2
        kbd.press_key(&C64Key::A);

        // When Port A = 0x00 (all columns selected simultaneously),
        // Row 1 must be grounded (bit 1 is 0 -> 0xFD)
        assert_eq!(
            kbd.scan_rows(0x00),
            0xFD,
            "Global poll 0x00 must show Row 1 grounded for key 'A'"
        );
    }

    // ---------------------------------------------------------------------
    // 5. Multi-Key Rollover & Modifiers (e.g. LeftShift + 1)
    // ---------------------------------------------------------------------

    #[test]
    fn simultaneous_left_shift_and_one_press() {
        // LeftShift: Row 1, Column 7
        // One:       Row 7, Column 0
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::LeftShift);
        kbd.press_key(&C64Key::One);

        // Scan Column 7 (LeftShift): only Row 1 low
        assert_eq!(kbd.scan_rows(!(1 << 7)), !(1 << 1));

        // Scan Column 0 (One): only Row 7 low
        assert_eq!(kbd.scan_rows(!(1 << 0)), !(1 << 7));

        // Scan both Column 7 and Column 0 simultaneously (bits 0 and 7 low):
        // Both Row 1 and Row 7 must drop low!
        let dual_col = !((1 << 7) | (1 << 0));
        let expected_rows = !((1 << 1) | (1 << 7));
        assert_eq!(kbd.scan_rows(dual_col), expected_rows);

        // Release 'One', LeftShift remains
        kbd.release_key(&C64Key::One);
        assert_eq!(kbd.scan_rows(dual_col), !(1 << 1));
    }

    #[test]
    fn multiple_keys_on_same_column() {
        // Del:    Row 0, Column 0
        // One:    Row 7, Column 0
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::Del);
        kbd.press_key(&C64Key::One);

        // Selecting Column 0 must drop BOTH Row 0 and Row 7!
        let expected_rows = !((1 << 0) | (1 << 7));
        assert_eq!(kbd.scan_rows(!(1 << 0)), expected_rows);
    }

    // ---------------------------------------------------------------------
    // 6. Reset & Idempotence
    // ---------------------------------------------------------------------

    #[test]
    fn double_press_and_unpressed_release_are_safe() {
        let mut kbd = Keyboard::new();

        // Releasing unpressed key does not panic or corrupt
        kbd.release_key(&C64Key::Return);
        assert_eq!(kbd.scan_rows(0x00), 0xFF);

        // Pressing twice
        kbd.press_key(&C64Key::Return);
        kbd.press_key(&C64Key::Return);
        assert_eq!(kbd.scan_rows(0x00), !(1 << 0)); // Return is Row 0, Col 1

        kbd.release_key(&C64Key::Return);
        assert_eq!(kbd.scan_rows(0x00), 0xFF);
    }

    #[test]
    fn reset_clears_all_pressed_keys() {
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::A);
        kbd.press_key(&C64Key::Space);
        kbd.press_key(&C64Key::LeftShift);
        assert_ne!(kbd.scan_rows(0x00), 0xFF);

        kbd.reset();
        assert_eq!(kbd.scan_rows(0x00), 0xFF);
        assert!(!kbd.is_pressed(&C64Key::A));
        assert!(!kbd.is_pressed(&C64Key::Space));
        assert!(!kbd.is_pressed(&C64Key::LeftShift));
    }

    // ---------------------------------------------------------------------
    // 7. Reverse Scanning (Port B rows drive Port A columns)
    // ---------------------------------------------------------------------

    #[test]
    fn reverse_scanning_rows_ground_columns() {
        // Space: Row 7, Column 4
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::Space);

        // Driving Row 7 low on Port B (0b0111_1111 = 0x7F)
        // Column 4 on Port A should drop low (0b1110_1111 = 0xEF)
        assert_eq!(
            kbd.scan_columns(0x7F),
            0xEF,
            "Driving Row 7 low must ground Column 4"
        );

        // Driving other rows low should not ground Column 4
        assert_eq!(kbd.scan_columns(0xFE), 0xFF);
    }

    // ---------------------------------------------------------------------
    // 8. Matrix data integrity — the tables themselves
    // ---------------------------------------------------------------------

    #[test]
    fn matrix_holds_exactly_64_distinct_keys() {
        let seen: HashSet<C64Key> = C64Key::ALL_ROWS.iter().flatten().copied().collect();
        assert_eq!(seen.len(), 64, "the matrix must contain 64 unique keys");

        let flat: HashSet<C64Key> = C64Key::ALL_KEYS.iter().copied().collect();
        assert_eq!(flat.len(), 64);
        assert_eq!(
            seen, flat,
            "ALL_KEYS and ALL_ROWS must describe the same keys"
        );
    }

    #[test]
    fn all_keys_is_row_major_flattening_of_all_rows() {
        let flattened: Vec<C64Key> = C64Key::ALL_ROWS.iter().flatten().copied().collect();
        assert_eq!(flattened.as_slice(), &C64Key::ALL_KEYS[..]);
    }

    #[test]
    fn row_constants_agree_with_all_rows() {
        assert_eq!(C64Key::ALL_ROWS[0], C64Key::ROW0);
        assert_eq!(C64Key::ALL_ROWS[1], C64Key::ROW1);
        assert_eq!(C64Key::ALL_ROWS[2], C64Key::ROW2);
        assert_eq!(C64Key::ALL_ROWS[3], C64Key::ROW3);
        assert_eq!(C64Key::ALL_ROWS[4], C64Key::ROW4);
        assert_eq!(C64Key::ALL_ROWS[5], C64Key::ROW5);
        assert_eq!(C64Key::ALL_ROWS[6], C64Key::ROW6);
        assert_eq!(C64Key::ALL_ROWS[7], C64Key::ROW7);
    }

    #[test]
    fn matrix_layout_matches_the_real_c64_reference_chart() {
        // Spot checks transcribed independently from the C64 key matrix
        // chart (Commodore 64 Programmer's Reference Guide). Rows are
        // $DC01 bits, columns are $DC00 bits. Off-diagonal coordinates
        // were chosen deliberately: they fail loudly if the matrix is
        // ever transposed (a square 8x8 transposition is otherwise
        // invisible to the symmetric scan logic!).
        let chart = [
            (C64Key::Del, 0, 0),
            (C64Key::CrsrHorizontal, 0, 2),
            (C64Key::F1, 0, 4),
            (C64Key::CrsrVertical, 0, 7),
            (C64Key::W, 1, 1),
            (C64Key::A, 1, 2),
            (C64Key::Z, 1, 4),
            (C64Key::LeftShift, 1, 7),
            (C64Key::Six, 2, 3),
            (C64Key::F, 2, 5),
            (C64Key::T, 2, 6),
            (C64Key::Seven, 3, 0),
            (C64Key::G, 3, 2),
            (C64Key::H, 3, 5),
            (C64Key::V, 3, 7),
            (C64Key::Zero, 4, 3),
            (C64Key::M, 4, 4),
            (C64Key::O, 4, 6),
            (C64Key::Plus, 5, 0),
            (C64Key::L, 5, 2),
            (C64Key::Colon, 5, 5),
            (C64Key::Comma, 5, 7),
            (C64Key::Pound, 6, 0),
            (C64Key::Home, 6, 3),
            (C64Key::RightShift, 6, 4),
            (C64Key::ArrowUp, 6, 6),
            (C64Key::One, 7, 0),
            (C64Key::Ctrl, 7, 2),
            (C64Key::Space, 7, 4),
            (C64Key::CEqual, 7, 5),
            (C64Key::RunStop, 7, 7),
        ];

        for (key, row, col) in chart {
            assert_eq!(
                key_at(row, col),
                key,
                "{key:?} must live at (row {row}, col {col}) per the C64 chart"
            );
        }
    }

    // ---------------------------------------------------------------------
    // 9. Exhaustive mask coverage — 64 keys x 256 masks, both directions
    // ---------------------------------------------------------------------

    #[test]
    fn idle_keyboard_reads_high_for_every_mask_in_both_directions() {
        let kbd = Keyboard::new();
        for mask in 0..=255u8 {
            assert_eq!(kbd.scan_rows(mask), 0xFF, "mask {mask:#04x}");
            assert_eq!(kbd.scan_columns(mask), 0xFF, "mask {mask:#04x}");
        }
    }

    #[test]
    fn every_single_key_scanned_over_all_256_column_masks() {
        for (row, keys) in C64Key::ALL_ROWS.iter().enumerate() {
            for (col, &key) in keys.iter().enumerate() {
                let mut kbd = Keyboard::new();
                kbd.press_key(&key);

                for mask in 0..=255u8 {
                    // Row `row` may only drop when column `col` is selected.
                    let expected = if mask & (1 << col) == 0 {
                        !(1 << row)
                    } else {
                        0xFF
                    };
                    assert_eq!(
                        kbd.scan_rows(mask),
                        expected,
                        "{key:?} at (row {row}, col {col}), column mask {mask:#04x}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_single_key_reverse_scanned_over_all_256_row_masks() {
        for (row, keys) in C64Key::ALL_ROWS.iter().enumerate() {
            for (col, &key) in keys.iter().enumerate() {
                let mut kbd = Keyboard::new();
                kbd.press_key(&key);

                for mask in 0..=255u8 {
                    let expected = if mask & (1 << row) == 0 {
                        !(1 << col)
                    } else {
                        0xFF
                    };
                    assert_eq!(
                        kbd.scan_columns(mask),
                        expected,
                        "{key:?} at (row {row}, col {col}), row mask {mask:#04x}"
                    );
                }
            }
        }
    }

    #[test]
    fn global_poll_detects_each_key_on_its_own_row_and_column() {
        for &key in &C64Key::ALL_KEYS {
            let (row, col) = coord_of(key);
            let mut kbd = Keyboard::new();
            kbd.press_key(&key);

            // $DC00 = $00: the KERNAL "any key down?" poll
            assert_eq!(kbd.scan_rows(0x00), !(1 << row), "{key:?}");
            // $DC01 = $00 driving every row: its column must drop
            assert_eq!(kbd.scan_columns(0x00), !(1 << col), "{key:?}");
        }
    }

    #[test]
    fn every_key_can_be_pressed_and_released_individually() {
        let mut kbd = Keyboard::new();
        for &key in &C64Key::ALL_KEYS {
            assert!(!kbd.is_pressed(&key));
            kbd.press_key(&key);
            assert!(kbd.is_pressed(&key));
            kbd.release_key(&key);
            assert!(!kbd.is_pressed(&key));
            assert_eq!(kbd.scan_rows(0x00), 0xFF);
            assert_eq!(kbd.scan_columns(0x00), 0xFF);
        }
    }

    #[test]
    fn pressing_one_key_never_reports_other_keys_as_pressed() {
        for &key in &C64Key::ALL_KEYS {
            let mut kbd = Keyboard::new();
            kbd.press_key(&key);
            let count = C64Key::ALL_KEYS
                .iter()
                .filter(|k| kbd.is_pressed(k))
                .count();
            assert_eq!(count, 1, "pressing {key:?} leaked into other keys");
        }
    }

    // ---------------------------------------------------------------------
    // 10. Keys sharing rows/columns, floods, full saturation
    // ---------------------------------------------------------------------

    #[test]
    fn two_keys_in_the_same_row_ground_both_columns_when_the_row_is_driven() {
        // A: (1, 2)   S: (1, 5) — same row
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &[C64Key::A, C64Key::S]);

        assert_eq!(kbd.scan_rows(!(1 << 2)), !(1 << 1));
        assert_eq!(kbd.scan_rows(!(1 << 5)), !(1 << 1));
        // Both columns selected at once — still only row 1
        assert_eq!(kbd.scan_rows(!((1 << 2) | (1 << 5))), !(1 << 1));
        // Unrelated column stays high
        assert_eq!(kbd.scan_rows(!(1 << 0)), 0xFF);
        assert_eq!(kbd.scan_rows(0x00), !(1 << 1));

        // Reverse direction: driving row 1 grounds columns 2 AND 5
        assert_eq!(kbd.scan_columns(!(1 << 1)), !((1 << 2) | (1 << 5)));
        assert_eq!(kbd.scan_columns(0x00), !((1 << 2) | (1 << 5)));
        assert_eq!(kbd.scan_columns(!(1 << 7)), 0xFF);
    }

    #[test]
    fn two_keys_in_the_same_column_ground_the_shared_column_from_either_row() {
        // Del: (0, 0)   One: (7, 0) — same column, reverse direction
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &[C64Key::Del, C64Key::One]);

        assert_eq!(kbd.scan_columns(!(1 << 0)), !(1 << 0));
        assert_eq!(kbd.scan_columns(!(1 << 7)), !(1 << 0));
        assert_eq!(kbd.scan_columns(!((1 << 0) | (1 << 7))), !(1 << 0));
        assert_eq!(kbd.scan_columns(!(1 << 3)), 0xFF); // unrelated row
        assert_eq!(kbd.scan_columns(0x00), 0xFE);
    }

    #[test]
    fn flooding_one_column_gounds_all_rows_only_for_that_column() {
        let mut kbd = Keyboard::new();
        let column_keys: Vec<C64Key> = (0..8).map(|row| key_at(row, 3)).collect();
        press_all(&mut kbd, &column_keys);

        assert_eq!(kbd.scan_rows(!(1 << 3)), 0x00);
        assert_eq!(kbd.scan_rows(0x00), 0x00);
        assert_eq!(kbd.scan_rows(0xFF), 0xFF);
        for col in [0u8, 1, 2, 4, 5, 6, 7] {
            assert_eq!(kbd.scan_rows(!(1 << col)), 0xFF, "col {col}");
        }
    }

    #[test]
    fn flooding_one_row_gounds_all_columns_only_for_that_row() {
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &C64Key::ROW2);

        assert_eq!(kbd.scan_columns(!(1 << 2)), 0x00);
        assert_eq!(kbd.scan_columns(0x00), 0x00);
        assert_eq!(kbd.scan_columns(0xFF), 0xFF);
        for row in 0..8 {
            if row != 2 {
                assert_eq!(kbd.scan_columns(!(1 << row)), 0xFF);
            }
        }
        // From the other direction: any column selection only grounds row 2
        for col in 0..8 {
            assert_eq!(kbd.scan_rows(!(1 << col)), !(1 << 2));
        }
    }

    #[test]
    fn pressing_all_64_keys_saturates_the_matrix() {
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &C64Key::ALL_KEYS);

        for &key in &C64Key::ALL_KEYS {
            assert!(kbd.is_pressed(&key));
        }
        assert_eq!(kbd.scan_rows(0x00), 0x00);
        assert_eq!(kbd.scan_columns(0x00), 0x00);
        // Unselected lines still float high
        assert_eq!(kbd.scan_rows(0xFF), 0xFF);
        assert_eq!(kbd.scan_columns(0xFF), 0xFF);
        for col in 0..8 {
            assert_eq!(kbd.scan_rows(!(1 << col)), 0x00);
        }
        for row in 0..8 {
            assert_eq!(kbd.scan_columns(!(1 << row)), 0x00);
        }

        // Partial release keeps exactly the 60 survivors
        for &key in &[C64Key::Del, C64Key::A, C64Key::Space, C64Key::RunStop] {
            kbd.release_key(&key);
        }
        let detected = kernal_column_walk(&kbd);
        assert_eq!(detected.len(), 60);
        assert!(!detected.contains(&C64Key::Del));
        assert!(!detected.contains(&C64Key::A));
        assert!(!detected.contains(&C64Key::Space));
        assert!(!detected.contains(&C64Key::RunStop));
    }

    // ---------------------------------------------------------------------
    // 11. Firmware-level round trips (KERNAL-style scan walks, typing)
    // ---------------------------------------------------------------------

    #[test]
    fn kernal_column_walk_recovers_every_single_key_exactly() {
        for &key in &C64Key::ALL_KEYS {
            let mut kbd = Keyboard::new();
            kbd.press_key(&key);
            assert_eq!(kernal_column_walk(&kbd), vec![key], "{key:?}");
        }
    }

    #[test]
    fn kernal_walk_and_reverse_walk_recover_exact_key_sets() {
        let cases: &[&[C64Key]] = &[
            &[],
            &[C64Key::Space],
            &[C64Key::LeftShift, C64Key::One], // shift combo, nothing shared
            &[C64Key::A, C64Key::S],           // same row
            &[C64Key::Del, C64Key::One],       // same column
            &[C64Key::E, C64Key::S, C64Key::T], // L-shape (ghost corner!)
            &[C64Key::LeftShift, C64Key::RunStop], // both in column 7
            &C64Key::ROW5,                     // a whole row
        ];

        for case in cases {
            let mut kbd = Keyboard::new();
            press_all(&mut kbd, case);

            let via_columns: HashSet<C64Key> = kernal_column_walk(&kbd).into_iter().collect();
            let via_rows: HashSet<C64Key> = reverse_row_walk(&kbd).into_iter().collect();
            let expected: HashSet<C64Key> = case.iter().copied().collect();

            assert_eq!(via_columns, expected, "KERNAL walk for {case:?}");
            assert_eq!(via_rows, expected, "reverse walk for {case:?}");
        }
    }

    #[test]
    fn held_modifier_while_tapping_keys() {
        // Simulate: hold LeftShift, tap A, then Z, then release shift.
        let mut kbd = Keyboard::new();
        kbd.press_key(&C64Key::LeftShift);

        for &key in &[C64Key::A, C64Key::Z] {
            kbd.press_key(&key);
            let detected = kernal_column_walk(&kbd);
            assert_eq!(detected.len(), 2);
            assert!(detected.contains(&C64Key::LeftShift));
            assert!(detected.contains(&key));
            kbd.release_key(&key);
            assert_eq!(kernal_column_walk(&kbd), vec![C64Key::LeftShift]);
        }

        kbd.release_key(&C64Key::LeftShift);
        assert!(kernal_column_walk(&kbd).is_empty());
    }

    #[test]
    fn typing_simulation_presses_scans_and_releases_each_key() {
        let typed = [
            C64Key::H,
            C64Key::E,
            C64Key::L,
            C64Key::L,
            C64Key::O,
            C64Key::Space,
            C64Key::W,
            C64Key::O,
            C64Key::R,
            C64Key::L,
            C64Key::D,
        ];

        let mut kbd = Keyboard::new();
        for &key in &typed {
            kbd.press_key(&key);
            assert_eq!(kernal_column_walk(&kbd), vec![key]);
            kbd.release_key(&key);
            assert!(kernal_column_walk(&kbd).is_empty());
            assert_eq!(kbd.scan_rows(0x00), 0xFF);
        }
    }

    // ---------------------------------------------------------------------
    // 12. Ghosting — this model is deliberately idealized
    //
    // The real C64 matrix has no diodes, so three corners of a rectangle
    // electrically ground the fourth corner ("ghost key"). This
    // implementation intentionally models a perfect n-key-rollover matrix
    // instead. These tests pin that decision; replace them if you ever
    // move to an electrically faithful (ghosting) model.
    // ---------------------------------------------------------------------

    #[test]
    fn three_corners_of_a_rectangle_do_not_ghost_the_fourth() {
        // E (1,6), S (1,5), T (2,6) held; the ghost corner would be F (2,5).
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &[C64Key::E, C64Key::S, C64Key::T]);

        // Selecting column 5 must ground ONLY row 1 (S); row 2 stays high
        // even though T (2,6) and E (1,6) are held.
        assert_eq!(kbd.scan_rows(!(1 << 5)), !(1 << 1));

        // Selecting column 6 grounds rows 1 and 2 (E and T).
        assert_eq!(kbd.scan_rows(!(1 << 6)), !((1 << 1) | (1 << 2)));

        let detected: HashSet<C64Key> = kernal_column_walk(&kbd).into_iter().collect();
        let expected: HashSet<C64Key> = [C64Key::E, C64Key::S, C64Key::T].into_iter().collect();
        assert_eq!(detected, expected, "no phantom 'F' may be reported");
    }

    #[test]
    fn no_ghost_keys_for_a_sample_of_l_shapes() {
        // Each tuple presses (r0,c0), (r0,c1), (r1,c1). On real hardware the
        // undiode'd matrix would also ground (r1,c0) when c0 is selected.
        let shapes = [
            (0, 0, 1, 1), // Del, Return, W   -> ghost would be '3'
            (4, 0, 5, 1), // '9', 'I', 'P'    -> ghost would be '+'
            (7, 4, 6, 5), // SPACE, C=, '='   -> ghost would be R-SHIFT
            (2, 6, 3, 7), // T, X, V          -> ghost would be U
        ];

        for (r0, c0, r1, c1) in shapes {
            let mut kbd = Keyboard::new();
            press_all(&mut kbd, &[key_at(r0, c0), key_at(r0, c1), key_at(r1, c1)]);

            // Scan c0: only row r0 may be low.
            assert_eq!(
                kbd.scan_rows(!(1 << c0)),
                !(1 << r0),
                "ghost detected at ({r1},{c0}) for shape ({r0},{c0},{r1},{c1})"
            );

            let detected: HashSet<C64Key> = kernal_column_walk(&kbd).into_iter().collect();
            let expected: HashSet<C64Key> = [key_at(r0, c0), key_at(r0, c1), key_at(r1, c1)]
                .into_iter()
                .collect();
            assert_eq!(detected, expected);
        }
    }

    #[test]
    fn all_four_corners_of_a_rectangle_ground_both_rows_and_columns() {
        // E (1,6), S (1,5), T (2,6), F (2,5) — the whole rectangle is real.
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &[C64Key::E, C64Key::S, C64Key::T, C64Key::F]);

        assert_eq!(kbd.scan_rows(!(1 << 5)), !((1 << 1) | (1 << 2)));
        assert_eq!(kbd.scan_rows(!(1 << 6)), !((1 << 1) | (1 << 2)));
        assert_eq!(kbd.scan_rows(0x00), !((1 << 1) | (1 << 2)));
        assert_eq!(kbd.scan_columns(!(1 << 1)), !((1 << 5) | (1 << 6)));
        assert_eq!(kbd.scan_columns(!(1 << 2)), !((1 << 5) | (1 << 6)));
        assert_eq!(kbd.scan_columns(0x00), !((1 << 5) | (1 << 6)));
    }

    // ---------------------------------------------------------------------
    // 13. Scanning is side-effect free; state persists across polls
    // ---------------------------------------------------------------------

    #[test]
    fn thousands_of_scans_never_consume_or_corrupt_key_state() {
        let mut kbd = Keyboard::new();
        press_all(&mut kbd, &[C64Key::G, C64Key::LeftShift]);

        let snapshot = (
            kbd.scan_rows(0x00),
            kbd.scan_rows(0xAA),
            kbd.scan_rows(0x55),
            kbd.scan_columns(0x00),
            kbd.scan_columns(0x5A),
        );
        for _ in 0..1000 {
            assert_eq!(
                (
                    kbd.scan_rows(0x00),
                    kbd.scan_rows(0xAA),
                    kbd.scan_rows(0x55),
                    kbd.scan_columns(0x00),
                    kbd.scan_columns(0x5A),
                ),
                snapshot
            );
        }
        assert!(kbd.is_pressed(&C64Key::G));
        assert!(kbd.is_pressed(&C64Key::LeftShift));
    }

    #[test]
    fn rapid_press_release_cycles_stay_consistent() {
        let mut kbd = Keyboard::new();
        for _ in 0..10 {
            kbd.press_key(&C64Key::Q); // Q: (7, 6)
            assert_eq!(kbd.scan_rows(0x00), !(1 << 7));
            assert_eq!(kbd.scan_rows(!(1 << 6)), !(1 << 7));
            kbd.release_key(&C64Key::Q);
            assert_eq!(kbd.scan_rows(0x00), 0xFF);
        }
    }

    // ---------------------------------------------------------------------
    // 14. Randomised differential testing (deterministic, no dependencies)
    // ---------------------------------------------------------------------

    #[test]
    fn fuzz_random_key_sets_agree_with_reference_model() {
        let mut rng = Rng(0x0C64_C0DE_C64);

        for _ in 0..500 {
            let target = rng.below(10) as usize; // 0..=9 keys
            let mut pressed: Vec<C64Key> = Vec::new();
            let mut seen = HashSet::new();
            while pressed.len() < target {
                let key = C64Key::ALL_KEYS[rng.below(64) as usize];
                if seen.insert(key) {
                    pressed.push(key);
                }
            }

            let mut kbd = Keyboard::new();
            press_all(&mut kbd, &pressed);

            for _ in 0..64 {
                let columns = rng.below(256) as u8;
                let rows = rng.below(256) as u8;
                assert_eq!(kbd.scan_rows(columns), reference_rows(&pressed, columns));
                assert_eq!(kbd.scan_columns(rows), reference_columns(&pressed, rows));
            }

            // Full KERNAL round-trip on the random chord.
            let detected: HashSet<C64Key> = kernal_column_walk(&kbd).into_iter().collect();
            let expected: HashSet<C64Key> = pressed.iter().copied().collect();
            assert_eq!(detected, expected);
        }
    }

    #[test]
    fn fuzz_random_press_release_reset_sequences_match_shadow_state() {
        let mut rng = Rng(0x5EED_0000_ABCD);
        let mut kbd = Keyboard::new();
        let mut shadow: HashSet<C64Key> = HashSet::new();

        for step in 0..2000 {
            let key = C64Key::ALL_KEYS[rng.below(64) as usize];
            let roll = rng.below(100);
            match roll {
                0..=44 => {
                    kbd.press_key(&key);
                    shadow.insert(key);
                }
                45..=89 => {
                    kbd.release_key(&key);
                    shadow.remove(&key);
                }
                90..=91 => {
                    kbd.reset();
                    shadow.clear();
                }
                _ => { /* pure scan step, no mutation */ }
            }

            // Full state agreement on every key, every step.
            for &k in &C64Key::ALL_KEYS {
                assert_eq!(
                    kbd.is_pressed(&k),
                    shadow.contains(&k),
                    "state mismatch for {k:?} at step {step}"
                );
            }

            // Random scans must agree with the reference model. Iteration
            // order of `shadow` doesn't matter: the model is a pure OR of
            // active-low bit clears.
            let pressed: Vec<C64Key> = shadow.iter().copied().collect();
            let columns = rng.below(256) as u8;
            let rows = rng.below(256) as u8;
            assert_eq!(
                kbd.scan_rows(columns),
                reference_rows(&pressed, columns),
                "step {step}"
            );
            assert_eq!(
                kbd.scan_columns(rows),
                reference_columns(&pressed, rows),
                "step {step}"
            );

            if step % 250 == 0 {
                let detected: HashSet<C64Key> = kernal_column_walk(&kbd).into_iter().collect();
                assert_eq!(detected, shadow, "walk mismatch at step {step}");
            }
        }
    }
}
