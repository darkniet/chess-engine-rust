use crate::board::Board;
use crate::pieces::{Color, PieceType};

/// Piece-square tables for positional evaluation
/// Values are from White's perspective, flipped for Black
const PAWN_TABLE: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

const KNIGHT_TABLE: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

const BISHOP_TABLE: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

const ROOK_TABLE: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

const QUEEN_TABLE: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

const KING_MIDDLE_TABLE: [[i32; 8]; 8] = [
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
];

const KING_END_TABLE: [[i32; 8]; 8] = [
    [-50, -40, -30, -20, -20, -30, -40, -50],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-50, -30, -30, -30, -30, -30, -30, -50],
];

pub struct Evaluator;

impl Evaluator {
    /// Evaluate the board position from the perspective of the given color
    /// Returns score in centipawns (positive = good for the color)
    pub fn evaluate(board: &Board, perspective: Color) -> i32 {
        let score = Self::evaluate_absolute(board);
        match perspective {
            Color::White => score,
            Color::Black => -score,
        }
    }

    /// Evaluate from absolute perspective, then flip for the given color
    /// More efficient than evaluate() when you already know whose turn it is
    pub fn evaluate_absolute_for(board: &Board, perspective: Color) -> i32 {
        let score = Self::evaluate_absolute(board);
        match perspective {
            Color::White => score,
            Color::Black => -score,
        }
    }

    /// Evaluate the board position (positive = good for white)
    pub fn evaluate_absolute(board: &Board) -> i32 {
        let mut score = 0;

        // Check for endgame
        let is_endgame = Self::is_endgame(board);

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get(rank, file) {
                    let piece_value = Self::piece_value(piece.piece_type);
                    let positional_value = Self::positional_value(
                        piece.piece_type,
                        piece.color,
                        rank,
                        file,
                        is_endgame,
                    );

                    let total = piece_value + positional_value;

                    match piece.color {
                        Color::White => score += total,
                        Color::Black => score -= total,
                    }
                }
            }
        }

        // Mobility bonus
        score += Self::mobility_bonus(board);

        score
    }

    fn piece_value(piece_type: PieceType) -> i32 {
        piece_type.value()
    }

    fn positional_value(
        piece_type: PieceType,
        color: Color,
        rank: usize,
        file: usize,
        is_endgame: bool,
    ) -> i32 {
        // For black, we need to flip the rank
        let table_rank = match color {
            Color::White => 7 - rank,
            Color::Black => rank,
        };

        let table = match piece_type {
            PieceType::Pawn => &PAWN_TABLE,
            PieceType::Knight => &KNIGHT_TABLE,
            PieceType::Bishop => &BISHOP_TABLE,
            PieceType::Rook => &ROOK_TABLE,
            PieceType::Queen => &QUEEN_TABLE,
            PieceType::King => {
                if is_endgame {
                    &KING_END_TABLE
                } else {
                    &KING_MIDDLE_TABLE
                }
            }
        };

        table[table_rank][file]
    }

    fn is_endgame(board: &Board) -> bool {
        let mut white_material = 0;
        let mut black_material = 0;

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get(rank, file) {
                    if piece.piece_type != PieceType::King && piece.piece_type != PieceType::Pawn {
                        match piece.color {
                            Color::White => white_material += piece.piece_type.value(),
                            Color::Black => black_material += piece.piece_type.value(),
                        }
                    }
                }
            }
        }

        // Endgame if both sides have less than a rook + minor piece worth of material
        white_material < 1300 && black_material < 1300
    }

    fn mobility_bonus(board: &Board) -> i32 {
        // Simple mobility: count pseudo-legal moves
        // This is an approximation, not counting all legal moves for speed
        let mut white_mobility = 0;
        let mut black_mobility = 0;

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get(rank, file) {
                    let mobility = Self::count_attacks(board, rank, file, piece.piece_type);
                    match piece.color {
                        Color::White => white_mobility += mobility,
                        Color::Black => black_mobility += mobility,
                    }
                }
            }
        }

        // 2 centipawns per square of mobility difference
        2 * (white_mobility - black_mobility)
    }

    fn count_attacks(board: &Board, rank: usize, file: usize, piece_type: PieceType) -> i32 {
        let mut count = 0;

        match piece_type {
            PieceType::Knight => {
                let offsets: [(i32, i32); 8] = [
                    (-2, -1),
                    (-2, 1),
                    (-1, -2),
                    (-1, 2),
                    (1, -2),
                    (1, 2),
                    (2, -1),
                    (2, 1),
                ];
                for (dr, df) in offsets {
                    let nr = rank as i32 + dr;
                    let nf = file as i32 + df;
                    if (0..8).contains(&nr) && (0..8).contains(&nf) {
                        count += 1;
                    }
                }
            }
            PieceType::Bishop => {
                for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                    let mut nr = rank as i32 + dr;
                    let mut nf = file as i32 + df;
                    while (0..8).contains(&nr) && (0..8).contains(&nf) {
                        count += 1;
                        if board.get(nr as usize, nf as usize).is_some() {
                            break;
                        }
                        nr += dr;
                        nf += df;
                    }
                }
            }
            PieceType::Rook => {
                for (dr, df) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let mut nr = rank as i32 + dr;
                    let mut nf = file as i32 + df;
                    while (0..8).contains(&nr) && (0..8).contains(&nf) {
                        count += 1;
                        if board.get(nr as usize, nf as usize).is_some() {
                            break;
                        }
                        nr += dr;
                        nf += df;
                    }
                }
            }
            PieceType::Queen => {
                for (dr, df) in [
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ] {
                    let mut nr = rank as i32 + dr;
                    let mut nf = file as i32 + df;
                    while (0..8).contains(&nr) && (0..8).contains(&nf) {
                        count += 1;
                        if board.get(nr as usize, nf as usize).is_some() {
                            break;
                        }
                        nr += dr;
                        nf += df;
                    }
                }
            }
            _ => {}
        }

        count
    }

    /// Get a large value representing checkmate
    pub const CHECKMATE_SCORE: i32 = 100000;
}
