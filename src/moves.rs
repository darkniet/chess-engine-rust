use crate::board::Board;
use crate::pieces::{Color, PieceType};

/// Represents a chess move
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from_rank: usize,
    pub from_file: usize,
    pub to_rank: usize,
    pub to_file: usize,
    pub promotion: Option<PieceType>,
    pub is_castling: bool,
    pub is_en_passant: bool,
}

impl Move {
    pub fn new(from_rank: usize, from_file: usize, to_rank: usize, to_file: usize) -> Self {
        Move {
            from_rank,
            from_file,
            to_rank,
            to_file,
            promotion: None,
            is_castling: false,
            is_en_passant: false,
        }
    }

    pub fn with_promotion(mut self, piece_type: PieceType) -> Self {
        self.promotion = Some(piece_type);
        self
    }

    pub fn as_castling(mut self) -> Self {
        self.is_castling = true;
        self
    }

    pub fn as_en_passant(mut self) -> Self {
        self.is_en_passant = true;
        self
    }
}

/// Move generator
pub struct MoveGenerator;

impl MoveGenerator {
    /// Generate all legal moves for the current player
    pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
        let pseudo_moves = Self::generate_pseudo_legal_moves(board);
        let mut legal_moves = Vec::new();

        for mv in pseudo_moves {
            let mut test_board = board.clone();
            test_board.make_move(mv);
            // Check if the move leaves our king in check
            if !test_board.is_in_check(board.current_turn) {
                legal_moves.push(mv);
            }
        }

        legal_moves
    }

    /// Generate all pseudo-legal moves (doesn't check for leaving king in check)
    pub fn generate_pseudo_legal_moves(board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let color = board.current_turn;

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get(rank, file) {
                    if piece.color == color {
                        match piece.piece_type {
                            PieceType::Pawn => {
                                Self::generate_pawn_moves(board, rank, file, &mut moves)
                            }
                            PieceType::Knight => {
                                Self::generate_knight_moves(board, rank, file, &mut moves)
                            }
                            PieceType::Bishop => {
                                Self::generate_bishop_moves(board, rank, file, &mut moves)
                            }
                            PieceType::Rook => {
                                Self::generate_rook_moves(board, rank, file, &mut moves)
                            }
                            PieceType::Queen => {
                                Self::generate_queen_moves(board, rank, file, &mut moves)
                            }
                            PieceType::King => {
                                Self::generate_king_moves(board, rank, file, &mut moves)
                            }
                        }
                    }
                }
            }
        }

        moves
    }

    fn generate_pawn_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let piece = board.get(rank, file).unwrap();
        let color = piece.color;
        let direction = color.pawn_direction();
        let start_rank = color.pawn_start_rank();
        let promotion_rank = color.promotion_rank();

        let new_rank = (rank as i32 + direction) as usize;

        // Forward move
        if new_rank < 8 && board.get(new_rank, file).is_none() {
            if new_rank == promotion_rank {
                // Promotion
                for promo in [
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ] {
                    moves.push(Move::new(rank, file, new_rank, file).with_promotion(promo));
                }
            } else {
                moves.push(Move::new(rank, file, new_rank, file));

                // Double move from starting position
                if rank == start_rank {
                    let double_rank = (rank as i32 + 2 * direction) as usize;
                    if board.get(double_rank, file).is_none() {
                        moves.push(Move::new(rank, file, double_rank, file));
                    }
                }
            }
        }

        // Captures
        for df in [-1i32, 1i32] {
            let new_file = file as i32 + df;
            if (0..8).contains(&new_file) {
                let new_file = new_file as usize;

                // Regular capture
                if let Some(target) = board.get(new_rank, new_file) {
                    if target.color != color {
                        if new_rank == promotion_rank {
                            for promo in [
                                PieceType::Queen,
                                PieceType::Rook,
                                PieceType::Bishop,
                                PieceType::Knight,
                            ] {
                                moves.push(
                                    Move::new(rank, file, new_rank, new_file).with_promotion(promo),
                                );
                            }
                        } else {
                            moves.push(Move::new(rank, file, new_rank, new_file));
                        }
                    }
                }

                // En passant
                if let Some((ep_rank, ep_file)) = board.en_passant {
                    if new_rank == ep_rank && new_file == ep_file {
                        moves.push(Move::new(rank, file, new_rank, new_file).as_en_passant());
                    }
                }
            }
        }
    }

    fn generate_knight_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let piece = board.get(rank, file).unwrap();
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
                let nr = nr as usize;
                let nf = nf as usize;
                if let Some(target) = board.get(nr, nf) {
                    if target.color != piece.color {
                        moves.push(Move::new(rank, file, nr, nf));
                    }
                } else {
                    moves.push(Move::new(rank, file, nr, nf));
                }
            }
        }
    }

    fn generate_sliding_moves(
        board: &Board,
        rank: usize,
        file: usize,
        directions: &[(i32, i32)],
        moves: &mut Vec<Move>,
    ) {
        let piece = board.get(rank, file).unwrap();

        for &(dr, df) in directions {
            let mut nr = rank as i32 + dr;
            let mut nf = file as i32 + df;

            while (0..8).contains(&nr) && (0..8).contains(&nf) {
                let nr_usize = nr as usize;
                let nf_usize = nf as usize;

                if let Some(target) = board.get(nr_usize, nf_usize) {
                    if target.color != piece.color {
                        moves.push(Move::new(rank, file, nr_usize, nf_usize));
                    }
                    break;
                } else {
                    moves.push(Move::new(rank, file, nr_usize, nf_usize));
                }

                nr += dr;
                nf += df;
            }
        }
    }

    fn generate_bishop_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        Self::generate_sliding_moves(board, rank, file, &directions, moves);
    }

    fn generate_rook_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        Self::generate_sliding_moves(board, rank, file, &directions, moves);
    }

    fn generate_queen_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let directions = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];
        Self::generate_sliding_moves(board, rank, file, &directions, moves);
    }

    fn generate_king_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let piece = board.get(rank, file).unwrap();

        // Normal king moves
        for dr in -1..=1 {
            for df in -1..=1 {
                if dr == 0 && df == 0 {
                    continue;
                }
                let nr = rank as i32 + dr;
                let nf = file as i32 + df;
                if (0..8).contains(&nr) && (0..8).contains(&nf) {
                    let nr = nr as usize;
                    let nf = nf as usize;
                    if let Some(target) = board.get(nr, nf) {
                        if target.color != piece.color {
                            moves.push(Move::new(rank, file, nr, nf));
                        }
                    } else {
                        moves.push(Move::new(rank, file, nr, nf));
                    }
                }
            }
        }

        // Castling
        Self::generate_castling_moves(board, rank, file, moves);
    }

    fn generate_castling_moves(board: &Board, rank: usize, file: usize, moves: &mut Vec<Move>) {
        let color = board.current_turn;
        let enemy = color.opposite();

        // Can't castle while in check
        if board.is_square_attacked(rank, file, enemy) {
            return;
        }

        let (kingside, queenside) = match color {
            Color::White => (
                board.castling_rights.white_kingside,
                board.castling_rights.white_queenside,
            ),
            Color::Black => (
                board.castling_rights.black_kingside,
                board.castling_rights.black_queenside,
            ),
        };

        // Kingside castling
        if kingside {
            // Check squares between king and rook are empty
            if board.get(rank, 5).is_none() && board.get(rank, 6).is_none() {
                // Check king doesn't pass through or end up in check
                if !board.is_square_attacked(rank, 5, enemy)
                    && !board.is_square_attacked(rank, 6, enemy)
                {
                    moves.push(Move::new(rank, file, rank, 6).as_castling());
                }
            }
        }

        // Queenside castling
        if queenside {
            // Check squares between king and rook are empty
            if board.get(rank, 1).is_none()
                && board.get(rank, 2).is_none()
                && board.get(rank, 3).is_none()
            {
                // Check king doesn't pass through or end up in check
                if !board.is_square_attacked(rank, 2, enemy)
                    && !board.is_square_attacked(rank, 3, enemy)
                {
                    moves.push(Move::new(rank, file, rank, 2).as_castling());
                }
            }
        }
    }

    /// Check if the current player is in checkmate
    pub fn is_checkmate(board: &Board) -> bool {
        board.is_in_check(board.current_turn) && Self::generate_legal_moves(board).is_empty()
    }

    /// Check if the current position is stalemate
    pub fn is_stalemate(board: &Board) -> bool {
        !board.is_in_check(board.current_turn) && Self::generate_legal_moves(board).is_empty()
    }

    /// Check for draw by insufficient material
    pub fn is_insufficient_material(board: &Board) -> bool {
        let mut white_pieces: Vec<PieceType> = Vec::new();
        let mut black_pieces: Vec<PieceType> = Vec::new();
        let mut white_bishops: Vec<(usize, usize)> = Vec::new();
        let mut black_bishops: Vec<(usize, usize)> = Vec::new();

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get(rank, file) {
                    match piece.color {
                        Color::White => {
                            white_pieces.push(piece.piece_type);
                            if piece.piece_type == PieceType::Bishop {
                                white_bishops.push((rank, file));
                            }
                        }
                        Color::Black => {
                            black_pieces.push(piece.piece_type);
                            if piece.piece_type == PieceType::Bishop {
                                black_bishops.push((rank, file));
                            }
                        }
                    }
                }
            }
        }

        let white_minor = white_pieces.iter().filter(|&&p| p != PieceType::King && p != PieceType::Pawn).count();
        let black_minor = black_pieces.iter().filter(|&&p| p != PieceType::King && p != PieceType::Pawn).count();

        // King vs King
        if white_minor == 0 && black_minor == 0 {
            return true;
        }

        // King + minor piece vs King
        if white_minor == 0 && black_minor == 1
            && (black_pieces.contains(&PieceType::Bishop) || black_pieces.contains(&PieceType::Knight))
        {
            return true;
        }
        if black_minor == 0 && white_minor == 1
            && (white_pieces.contains(&PieceType::Bishop) || white_pieces.contains(&PieceType::Knight))
        {
            return true;
        }

        // King + Bishop vs King + Bishop (same color squares)
        if white_pieces.len() == 2 && black_pieces.len() == 2 {
            if let (Some(_), Some(_)) = (white_pieces.iter().find(|&&p| p == PieceType::Bishop),
                                          black_pieces.iter().find(|&&p| p == PieceType::Bishop)) {
                if white_bishops.len() == 1 && black_bishops.len() == 1 {
                    let wb = white_bishops[0];
                    let bb = black_bishops[0];
                    // Check if both bishops are on the same color square
                    let wb_color = (wb.0 + wb.1) % 2;
                    let bb_color = (bb.0 + bb.1) % 2;
                    if wb_color == bb_color {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check for draw by fifty-move rule
    pub fn is_fifty_move_draw(board: &Board) -> bool {
        board.halfmove_clock >= 100
    }
}
