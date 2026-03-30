/// Color of a chess piece
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite color
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the direction pawns move (1 for white, -1 for black)
    pub fn pawn_direction(self) -> i32 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /// Returns the starting rank for pawns (1 for white, 6 for black)
    pub fn pawn_start_rank(self) -> usize {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }

    /// Returns the promotion rank for pawns (7 for white, 0 for black)
    pub fn promotion_rank(self) -> usize {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

/// Type of chess piece
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    /// Returns the material value of the piece (in centipawns)
    pub fn value(self) -> i32 {
        match self {
            PieceType::King => 20000,
            PieceType::Queen => 900,
            PieceType::Rook => 500,
            PieceType::Bishop => 330,
            PieceType::Knight => 320,
            PieceType::Pawn => 100,
        }
    }
}

/// A chess piece with color and type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    pub fn new(color: Color, piece_type: PieceType) -> Self {
        Piece { color, piece_type }
    }

    /// Returns the Unicode symbol for this piece
    pub fn symbol(self) -> char {
        match (self.color, self.piece_type) {
            (Color::White, PieceType::King) => '\u{2654}',
            (Color::White, PieceType::Queen) => '\u{2655}',
            (Color::White, PieceType::Rook) => '\u{2656}',
            (Color::White, PieceType::Bishop) => '\u{2657}',
            (Color::White, PieceType::Knight) => '\u{2658}',
            (Color::White, PieceType::Pawn) => '\u{2659}',
            (Color::Black, PieceType::King) => '\u{265A}',
            (Color::Black, PieceType::Queen) => '\u{265B}',
            (Color::Black, PieceType::Rook) => '\u{265C}',
            (Color::Black, PieceType::Bishop) => '\u{265D}',
            (Color::Black, PieceType::Knight) => '\u{265E}',
            (Color::Black, PieceType::Pawn) => '\u{265F}',
        }
    }
}
