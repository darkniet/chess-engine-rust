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

/// Search result for a position
enum SearchResult {
    /// Normal score
    Score(i32),
    /// Time ran out, result is unreliable
    Timeout,
}

/// Chess bot using negamax with alpha-beta pruning
pub struct ChessBot {
    difficulty: BotDifficulty,
    nodes_searched: u64,
    time_limit: Duration,
    timed_out: bool,
}

impl ChessBot {
    pub fn new(difficulty: BotDifficulty) -> Self {
        ChessBot {
            difficulty,
            nodes_searched: 0,
            time_limit: Duration::from_secs(10),
            timed_out: false,
        }
    }

    pub fn set_time_limit(&mut self, limit: Duration) {
        self.time_limit = limit;
    }

    /// Find the best move for the current position
    pub fn find_best_move(&mut self, board: &Board) -> Option<Move> {
        self.nodes_searched = 0;
        self.timed_out = false;
        let start_time = Instant::now();

        let moves = MoveGenerator::generate_legal_moves(board);
        if moves.is_empty() {
            return None;
        }

        let depth = self.difficulty.depth();

        let mut best_move = moves[0];
        let mut best_score = i32::MIN + 1;

        // Order moves for better pruning (captures first)
        let mut ordered_moves = moves;
        Self::order_moves(board, &mut ordered_moves);

        let mut board = board.clone();

        for mv in ordered_moves {
            board.make_move(mv);
            let result = self.negamax(
                &mut board,
                depth - 1,
                i32::MIN + 1,
                i32::MAX,
                start_time,
            );
            board.undo_move();

            if self.timed_out {
                break;
            }

            let score = match result {
                SearchResult::Score(s) => -s,
                SearchResult::Timeout => continue,
            };

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        Some(best_move)
    }

    /// Negamax with alpha-beta pruning (uses make/undo instead of cloning)
    fn negamax(
        &mut self,
        board: &mut Board,
        depth: u32,
        mut alpha: i32,
        beta: i32,
        start_time: Instant,
    ) -> SearchResult {
        self.nodes_searched += 1;

        // Time check every 1024 nodes to reduce overhead
        if self.nodes_searched & 1023 == 0 && start_time.elapsed() > self.time_limit {
            self.timed_out = true;
            return SearchResult::Timeout;
        }

        if self.timed_out {
            return SearchResult::Timeout;
        }

        // Generate legal moves once (not 3 times like before)
        let moves = MoveGenerator::generate_legal_moves(board);

        if moves.is_empty() {
            if board.is_in_check(board.current_turn) {
                return SearchResult::Score(-(Evaluator::CHECKMATE_SCORE + depth as i32));
            }
            return SearchResult::Score(0); // Stalemate
        }

        // Draw checks
        if MoveGenerator::is_insufficient_material(board)
            || MoveGenerator::is_fifty_move_draw(board)
        {
            return SearchResult::Score(0);
        }

        // Leaf node — use quiescence search
        if depth == 0 {
            let score = self.quiescence_search(board, alpha, beta, 4, start_time);
            return match score {
                SearchResult::Score(s) => SearchResult::Score(s),
                SearchResult::Timeout => SearchResult::Timeout,
            };
        }

        // Order moves for better pruning
        let mut ordered_moves = moves;
        Self::order_moves(board, &mut ordered_moves);

        for mv in ordered_moves {
            board.make_move(mv);
            let result = self.negamax(board, depth - 1, -beta, -alpha, start_time);
            board.undo_move();

            if let SearchResult::Timeout = result {
                return SearchResult::Timeout;
            }

            let score = -match result {
                SearchResult::Score(s) => s,
                _ => unreachable!(),
            };

            if score >= beta {
                return SearchResult::Score(beta); // Beta cutoff
            }
            if score > alpha {
                alpha = score;
            }
        }

        SearchResult::Score(alpha)
    }

    /// Quiescence search to handle tactical positions (captures only)
    fn quiescence_search(
        &mut self,
        board: &mut Board,
        mut alpha: i32,
        beta: i32,
        depth: i32,
        start_time: Instant,
    ) -> SearchResult {
        self.nodes_searched += 1;

        if self.timed_out || depth <= 0 {
            return SearchResult::Score(Evaluator::evaluate_absolute_for(board, board.current_turn));
        }

        // Time check
        if self.nodes_searched & 1023 == 0 && start_time.elapsed() > self.time_limit {
            self.timed_out = true;
            return SearchResult::Timeout;
        }

        let stand_pat = Evaluator::evaluate_absolute_for(board, board.current_turn);

        if stand_pat >= beta {
            return SearchResult::Score(beta);
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Only search captures
        let moves = MoveGenerator::generate_legal_moves(board);
        let mut captures: Vec<Move> = moves
            .into_iter()
            .filter(|mv| {
                board.get(mv.to_rank, mv.to_file).is_some() || mv.is_en_passant
            })
            .collect();

        // Order captures by MVV-LVA
        Self::order_moves(board, &mut captures);

        for mv in captures {
            board.make_move(mv);
            let result = self.quiescence_search(board, -beta, -alpha, depth - 1, start_time);
            board.undo_move();

            if let SearchResult::Timeout = result {
                return SearchResult::Timeout;
            }

            let score = -match result {
                SearchResult::Score(s) => s,
                _ => unreachable!(),
            };

            if score >= beta {
                return SearchResult::Score(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }

        SearchResult::Score(alpha)
    }

    /// Order moves to improve alpha-beta pruning
    fn order_moves(board: &Board, moves: &mut [Move]) {
        moves.sort_by_key(|mv| {
            let mut score = 0;

            // Captures are good (MVV-LVA: Most Valuable Victim - Least Valuable Attacker)
            if let Some(captured) = board.get(mv.to_rank, mv.to_file) {
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
