use crate::moves::Move;
use crate::pieces::{Color, Piece, PieceType};

/// Castling rights for both sides
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

/// State saved before a move for undoing
#[derive(Debug, Clone)]
pub struct UndoInfo {
    pub captured_piece: Option<Piece>,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<(usize, usize)>,
    pub halfmove_clock: u32,
    pub king_positions: [Option<(usize, usize)>; 2],
}

/// The chess board state
#[derive(Debug, Clone)]
pub struct Board {
    /// 8x8 board: board[rank][file], rank 0 = row 1, file 0 = column a
    pub squares: [[Option<Piece>; 8]; 8],
    pub current_turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<(usize, usize)>, // Square that can be captured en passant
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub undo_stack: Vec<(Move, UndoInfo)>,
    /// Cached king positions: [White index, Black index]
    pub king_positions: [Option<(usize, usize)>; 2],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Creates a new board with the starting position
    pub fn new() -> Self {
        let mut board = Board {
            squares: [[None; 8]; 8],
            current_turn: Color::White,
            castling_rights: CastlingRights::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            undo_stack: Vec::new(),
            king_positions: [Some((0, 4)), Some((7, 4))],
        };

        // Place white pieces
        board.squares[0][0] = Some(Piece::new(Color::White, PieceType::Rook));
        board.squares[0][1] = Some(Piece::new(Color::White, PieceType::Knight));
        board.squares[0][2] = Some(Piece::new(Color::White, PieceType::Bishop));
        board.squares[0][3] = Some(Piece::new(Color::White, PieceType::Queen));
        board.squares[0][4] = Some(Piece::new(Color::White, PieceType::King));
        board.squares[0][5] = Some(Piece::new(Color::White, PieceType::Bishop));
        board.squares[0][6] = Some(Piece::new(Color::White, PieceType::Knight));
        board.squares[0][7] = Some(Piece::new(Color::White, PieceType::Rook));
        for file in 0..8 {
            board.squares[1][file] = Some(Piece::new(Color::White, PieceType::Pawn));
        }

        // Place black pieces
        board.squares[7][0] = Some(Piece::new(Color::Black, PieceType::Rook));
        board.squares[7][1] = Some(Piece::new(Color::Black, PieceType::Knight));
        board.squares[7][2] = Some(Piece::new(Color::Black, PieceType::Bishop));
        board.squares[7][3] = Some(Piece::new(Color::Black, PieceType::Queen));
        board.squares[7][4] = Some(Piece::new(Color::Black, PieceType::King));
        board.squares[7][5] = Some(Piece::new(Color::Black, PieceType::Bishop));
        board.squares[7][6] = Some(Piece::new(Color::Black, PieceType::Knight));
        board.squares[7][7] = Some(Piece::new(Color::Black, PieceType::Rook));
        for file in 0..8 {
            board.squares[6][file] = Some(Piece::new(Color::Black, PieceType::Pawn));
        }

        board
    }

    /// Get the piece at a given square
    pub fn get(&self, rank: usize, file: usize) -> Option<Piece> {
        self.squares[rank][file]
    }

    /// Set a piece at a given square
    pub fn set(&mut self, rank: usize, file: usize, piece: Option<Piece>) {
        self.squares[rank][file] = piece;
    }

    /// Get the color index for king_positions array (0=White, 1=Black)
    #[inline]
    fn color_index(color: Color) -> usize {
        match color {
            Color::White => 0,
            Color::Black => 1,
        }
    }

    /// Find the king's position for a given color (O(1) with cache)
    pub fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        self.king_positions[Self::color_index(color)]
    }

    /// Update the cached king position
    fn update_king_position(&mut self, color: Color, rank: usize, file: usize) {
        self.king_positions[Self::color_index(color)] = Some((rank, file));
    }

    /// Check if a square is attacked by a given color
    pub fn is_square_attacked(&self, rank: usize, file: usize, by_color: Color) -> bool {
        // Check knight attacks
        let knight_offsets: [(i32, i32); 8] = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];
        for (dr, df) in knight_offsets {
            let nr = rank as i32 + dr;
            let nf = file as i32 + df;
            if (0..8).contains(&nr) && (0..8).contains(&nf) {
                if let Some(piece) = self.squares[nr as usize][nf as usize] {
                    if piece.color == by_color && piece.piece_type == PieceType::Knight {
                        return true;
                    }
                }
            }
        }

        // Check king attacks
        for dr in -1..=1 {
            for df in -1..=1 {
                if dr == 0 && df == 0 {
                    continue;
                }
                let nr = rank as i32 + dr;
                let nf = file as i32 + df;
                if (0..8).contains(&nr) && (0..8).contains(&nf) {
                    if let Some(piece) = self.squares[nr as usize][nf as usize] {
                        if piece.color == by_color && piece.piece_type == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }

        // Check pawn attacks
        let pawn_dir = if by_color == Color::White { -1 } else { 1 };
        for df in [-1, 1] {
            let nr = rank as i32 + pawn_dir;
            let nf = file as i32 + df;
            if (0..8).contains(&nr) && (0..8).contains(&nf) {
                if let Some(piece) = self.squares[nr as usize][nf as usize] {
                    if piece.color == by_color && piece.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }

        // Check sliding pieces (rook, bishop, queen)
        // Rook/Queen directions
        let rook_dirs: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dr, df) in rook_dirs {
            let mut nr = rank as i32 + dr;
            let mut nf = file as i32 + df;
            while (0..8).contains(&nr) && (0..8).contains(&nf) {
                if let Some(piece) = self.squares[nr as usize][nf as usize] {
                    if piece.color == by_color
                        && (piece.piece_type == PieceType::Rook
                            || piece.piece_type == PieceType::Queen)
                    {
                        return true;
                    }
                    break;
                }
                nr += dr;
                nf += df;
            }
        }

        // Bishop/Queen directions
        let bishop_dirs: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        for (dr, df) in bishop_dirs {
            let mut nr = rank as i32 + dr;
            let mut nf = file as i32 + df;
            while (0..8).contains(&nr) && (0..8).contains(&nf) {
                if let Some(piece) = self.squares[nr as usize][nf as usize] {
                    if piece.color == by_color
                        && (piece.piece_type == PieceType::Bishop
                            || piece.piece_type == PieceType::Queen)
                    {
                        return true;
                    }
                    break;
                }
                nr += dr;
                nf += df;
            }
        }

        false
    }

    /// Check if the given color's king is in check (uses cached king position)
    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some((rank, file)) = self.find_king(color) {
            self.is_square_attacked(rank, file, color.opposite())
        } else {
            false
        }
    }

    /// Make a move on the board
    pub fn make_move(&mut self, mv: Move) {
        let undo_info = UndoInfo {
            captured_piece: self.squares[mv.to_rank][mv.to_file],
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            king_positions: self.king_positions,
        };

        let piece = self.squares[mv.from_rank][mv.from_file].unwrap();

        // Handle en passant capture
        if mv.is_en_passant {
            let captured_rank = if piece.color == Color::White {
                mv.to_rank - 1
            } else {
                mv.to_rank + 1
            };
            self.squares[captured_rank][mv.to_file] = None;
        }

        // Handle castling
        if mv.is_castling {
            let rook_from_file;
            let rook_to_file;
            if mv.to_file > mv.from_file {
                // Kingside
                rook_from_file = 7;
                rook_to_file = 5;
            } else {
                // Queenside
                rook_from_file = 0;
                rook_to_file = 3;
            }
            let rook = self.squares[mv.from_rank][rook_from_file];
            self.squares[mv.from_rank][rook_from_file] = None;
            self.squares[mv.from_rank][rook_to_file] = rook;
        }

        // Move the piece
        self.squares[mv.from_rank][mv.from_file] = None;
        if let Some(promo) = mv.promotion {
            self.squares[mv.to_rank][mv.to_file] = Some(Piece::new(piece.color, promo));
        } else {
            self.squares[mv.to_rank][mv.to_file] = Some(piece);
        }

        // Update cached king position
        if piece.piece_type == PieceType::King {
            self.update_king_position(piece.color, mv.to_rank, mv.to_file);
        }

        // Update en passant square
        self.en_passant = None;
        if piece.piece_type == PieceType::Pawn {
            let rank_diff = (mv.to_rank as i32 - mv.from_rank as i32).abs();
            if rank_diff == 2 {
                let ep_rank = (mv.from_rank + mv.to_rank) / 2;
                self.en_passant = Some((ep_rank, mv.to_file));
            }
        }

        // Update castling rights
        if piece.piece_type == PieceType::King {
            if piece.color == Color::White {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }
        if piece.piece_type == PieceType::Rook {
            if mv.from_rank == 0 && mv.from_file == 0 {
                self.castling_rights.white_queenside = false;
            } else if mv.from_rank == 0 && mv.from_file == 7 {
                self.castling_rights.white_kingside = false;
            } else if mv.from_rank == 7 && mv.from_file == 0 {
                self.castling_rights.black_queenside = false;
            } else if mv.from_rank == 7 && mv.from_file == 7 {
                self.castling_rights.black_kingside = false;
            }
        }
        // Also update if a rook is captured
        if mv.to_rank == 0 && mv.to_file == 0 {
            self.castling_rights.white_queenside = false;
        } else if mv.to_rank == 0 && mv.to_file == 7 {
            self.castling_rights.white_kingside = false;
        } else if mv.to_rank == 7 && mv.to_file == 0 {
            self.castling_rights.black_queenside = false;
        } else if mv.to_rank == 7 && mv.to_file == 7 {
            self.castling_rights.black_kingside = false;
        }

        // Update halfmove clock
        if piece.piece_type == PieceType::Pawn || undo_info.captured_piece.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Update fullmove number
        if self.current_turn == Color::Black {
            self.fullmove_number += 1;
        }

        // Switch turn
        self.current_turn = self.current_turn.opposite();

        // Save undo info
        self.undo_stack.push((mv, undo_info));
    }

    /// Undo the last move
    pub fn undo_move(&mut self) {
        if let Some((mv, undo_info)) = self.undo_stack.pop() {
            // Switch turn back
            self.current_turn = self.current_turn.opposite();

            // Get the piece that was moved
            let mut piece = self.squares[mv.to_rank][mv.to_file].unwrap();

            // Handle promotion - restore to pawn
            if mv.promotion.is_some() {
                piece = Piece::new(piece.color, PieceType::Pawn);
            }

            // Move piece back
            self.squares[mv.from_rank][mv.from_file] = Some(piece);
            self.squares[mv.to_rank][mv.to_file] = undo_info.captured_piece;

            // Handle en passant
            if mv.is_en_passant {
                let captured_rank = if piece.color == Color::White {
                    mv.to_rank - 1
                } else {
                    mv.to_rank + 1
                };
                self.squares[captured_rank][mv.to_file] =
                    Some(Piece::new(piece.color.opposite(), PieceType::Pawn));
            }

            // Handle castling
            if mv.is_castling {
                let rook_from_file;
                let rook_to_file;
                if mv.to_file > mv.from_file {
                    rook_from_file = 7;
                    rook_to_file = 5;
                } else {
                    rook_from_file = 0;
                    rook_to_file = 3;
                }
                let rook = self.squares[mv.from_rank][rook_to_file];
                self.squares[mv.from_rank][rook_to_file] = None;
                self.squares[mv.from_rank][rook_from_file] = rook;
            }

            // Restore state (including cached king positions)
            self.castling_rights = undo_info.castling_rights;
            self.en_passant = undo_info.en_passant;
            self.halfmove_clock = undo_info.halfmove_clock;
            self.king_positions = undo_info.king_positions;

            if self.current_turn == Color::Black {
                self.fullmove_number -= 1;
            }
        }
    }
}
