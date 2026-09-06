#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum C64Key {
    // Row 0
    Del,
    Return,
    CrsrHorizontal,
    F7,
    F1,
    F3,
    F5,
    CrsrVertical,
    // Row 1
    Three,
    W,
    A,
    Four,
    Z,
    S,
    E,
    LeftShift,
    // Row 2
    Five,
    R,
    D,
    Six,
    C,
    F,
    T,
    X,
    // Row 3
    Seven,
    Y,
    G,
    Eight,
    B,
    H,
    U,
    V,
    // Row 4
    Nine,
    I,
    J,
    Zero,
    M,
    K,
    O,
    N,
    // Row 5
    Plus,
    P,
    L,
    Minus,
    Dot,
    Colon,
    At,
    Comma,
    // Row 6
    Pound,
    Star,
    Semicolon,
    Home,
    RightShift,
    Equal,
    ArrowUp,
    Slash,
    // Row 7
    One,
    ArrowLeft,
    Ctrl,
    Two,
    Space,
    CEqual,
    Q,
    RunStop,
}

impl C64Key {
    pub const ROW0: [Self; 8] = [
        Self::Del,
        Self::Return,
        Self::CrsrHorizontal,
        Self::F7,
        Self::F1,
        Self::F3,
        Self::F5,
        Self::CrsrVertical,
    ];

    pub const ROW1: [Self; 8] = [
        Self::Three,
        Self::W,
        Self::A,
        Self::Four,
        Self::Z,
        Self::S,
        Self::E,
        Self::LeftShift,
    ];

    pub const ROW2: [Self; 8] = [
        Self::Five,
        Self::R,
        Self::D,
        Self::Six,
        Self::C,
        Self::F,
        Self::T,
        Self::X,
    ];

    pub const ROW3: [Self; 8] = [
        Self::Seven,
        Self::Y,
        Self::G,
        Self::Eight,
        Self::B,
        Self::H,
        Self::U,
        Self::V,
    ];

    pub const ROW4: [Self; 8] = [
        Self::Nine,
        Self::I,
        Self::J,
        Self::Zero,
        Self::M,
        Self::K,
        Self::O,
        Self::N,
    ];

    pub const ROW5: [Self; 8] = [
        Self::Plus,
        Self::P,
        Self::L,
        Self::Minus,
        Self::Dot,
        Self::Colon,
        Self::At,
        Self::Comma,
    ];

    pub const ROW6: [Self; 8] = [
        Self::Pound,
        Self::Star,
        Self::Semicolon,
        Self::Home,
        Self::RightShift,
        Self::Equal,
        Self::ArrowUp,
        Self::Slash,
    ];

    pub const ROW7: [Self; 8] = [
        Self::One,
        Self::ArrowLeft,
        Self::Ctrl,
        Self::Two,
        Self::Space,
        Self::CEqual,
        Self::Q,
        Self::RunStop,
    ];

    pub const ALL_ROWS: [[Self; 8]; 8] = [
        Self::ROW0,
        Self::ROW1,
        Self::ROW2,
        Self::ROW3,
        Self::ROW4,
        Self::ROW5,
        Self::ROW6,
        Self::ROW7,
    ];

    pub const ALL_KEYS: [Self; 64] = {
        let mut flat = [Self::Del; 64];

        let matrix = [
            Self::ROW0,
            Self::ROW1,
            Self::ROW2,
            Self::ROW3,
            Self::ROW4,
            Self::ROW5,
            Self::ROW6,
            Self::ROW7,
        ];

        let mut idx = 0;
        let mut row = 0;
        while row < matrix.len() {
            let mut col = 0;

            while col < matrix[row].len() {
                flat[idx] = matrix[row][col];
                idx += 1;
                col += 1;
            }

            row += 1;
        }

        flat
    };
}
