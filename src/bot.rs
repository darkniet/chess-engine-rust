use crate::board::Board;
use crate::evaluation::Evaluator;
use crate::moves::{Move, MoveGenerator};
use crate::pieces::Color;
use std::time::{Duration, Instant};

/// Chess bot difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotDifficulty {
    Easy,   // Depth 2
    Medium, // Depth 3
    Hard,   // Depth 4
}

impl BotDifficulty {
    pub fn depth(self) -> u32 {
        match self {
            BotDifficulty::Easy => 2,
            BotDifficulty::Medium => 3,
            BotDifficulty::Hard => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BotDifficulty::Easy => "Easy",
            BotDifficulty::Medium => "Medium",
            BotDifficulty::Hard => "Hard",
        }
    }
}

/// Chess bot using minimax with alpha-beta pruning
pub struct ChessBot {
    difficulty: BotDifficulty,
    nodes_searched: u64,
    time_limit: Duration,
}

impl ChessBot {
    pub fn new(difficulty: BotDifficulty) -> Self {
        ChessBot {
            difficulty,
            nodes_searched: 0,
            time_limit: Duration::from_secs(10),
        }
    }

    pub fn set_time_limit(&mut self, limit: Duration) {
        self.time_limit = limit;
    }

    /// Find the best move for the current position
    pub fn find_best_move(&mut self, board: &Board) -> Option<Move> {
        self.nodes_searched = 0;
        let start_time = Instant::now();

        let moves = MoveGenerator::generate_legal_moves(board);
        if moves.is_empty() {
            return None;
        }

        let color = board.current_turn;
        let depth = self.difficulty.depth();

        let mut best_move = moves[0];
        let mut best_score = i32::MIN + 1;

        // Order moves for better pruning (captures first)
        let mut ordered_moves = moves;
        Self::order_moves(board, &mut ordered_moves);

        for mv in ordered_moves {
            let mut new_board = board.clone();
            new_board.make_move(mv);

            let score = -self.negamax(
                &mut new_board,
                depth - 1,
                i32::MIN + 1,
                i32::MAX,
                color.opposite(),
                start_time,
            );

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            // Time check
            if start_time.elapsed() > self.time_limit {
                break;
            }
        }

        Some(best_move)
    }

    /// Negamax with alpha-beta pruning
    fn negamax(
        &mut self,
        board: &mut Board,
        depth: u32,
        mut alpha: i32,
        beta: i32,
        color: Color,
        start_time: Instant,
    ) -> i32 {
        self.nodes_searched += 1;

        // Time check
        if start_time.elapsed() > self.time_limit {
            return 0;
        }

        // Check for terminal positions
        if MoveGenerator::is_checkmate(board) {
            return -Evaluator::CHECKMATE_SCORE - depth as i32; // Prefer faster checkmates
        }

        if MoveGenerator::is_stalemate(board)
            || MoveGenerator::is_insufficient_material(board)
            || MoveGenerator::is_fifty_move_draw(board)
        {
            return 0; // Draw
        }

        // Leaf node
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta, color, 4, start_time);
        }

        let moves = MoveGenerator::generate_legal_moves(board);
        if moves.is_empty() {
            if board.is_in_check(color) {
                return -Evaluator::CHECKMATE_SCORE - depth as i32;
            }
            return 0; // Stalemate
        }

        // Order moves for better pruning
        let mut ordered_moves = moves;
        Self::order_moves(board, &mut ordered_moves);

        for mv in ordered_moves {
            board.make_move(mv);
            let score = -self.negamax(board, depth - 1, -beta, -alpha, color.opposite(), start_time);
            board.undo_move();

            if score >= beta {
                return beta; // Beta cutoff
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Quiescence search to handle tactical positions
    fn quiescence_search(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        color: Color,
        depth: i32,
        start_time: Instant,
    ) -> i32 {
        self.nodes_searched += 1;

        if start_time.elapsed() > self.time_limit || depth <= 0 {
            return Evaluator::evaluate(board, color);
        }

        let stand_pat = Evaluator::evaluate(board, color);

        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Only search captures
        let moves = MoveGenerator::generate_legal_moves(board);
        let captures: Vec<Move> = moves
            .into_iter()
            .filter(|mv| {
                board.get(mv.to_rank, mv.to_file).is_some() || mv.is_en_passant
            })
            .collect();

        for mv in captures {
            board.make_move(mv);
            let score = -self.quiescence_search(board, -beta, -alpha, color.opposite(), depth - 1, start_time);
            board.undo_move();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Order moves to improve alpha-beta pruning
    fn order_moves(board: &Board, moves: &mut [Move]) {
        moves.sort_by_key(|mv| {
            let mut score = 0;

            // Captures are good
            if let Some(captured) = board.get(mv.to_rank, mv.to_file) {
                // MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
                let attacker = board.get(mv.from_rank, mv.from_file).unwrap();
                score -= 10 * captured.piece_type.value() - attacker.piece_type.value();
            }

            // Promotions are good
            if mv.promotion.is_some() {
                score -= 800;
            }

            score
        });
    }

    pub fn nodes_searched(&self) -> u64 {
        self.nodes_searched
    }
}
