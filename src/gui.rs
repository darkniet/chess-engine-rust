use crate::board::Board;
use crate::bot::{BotDifficulty, ChessBot};
use crate::moves::{Move, MoveGenerator};
use crate::pieces::{Color, Piece, PieceType};
use crate::timer::{format_time, GameTimer, TimeControl};
use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::time::Instant;

/// Game mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    VsBot,
    LocalPvP,
}

/// Game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    ModeSelect,
    Playing,
    GameOver(GameResult),
}

/// Game result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

// Color scheme
fn bg_dark() -> Color32 {
    Color32::from_rgb(30, 30, 35)
}
fn bg_panel() -> Color32 {
    Color32::from_rgb(45, 45, 50)
}
fn accent() -> Color32 {
    Color32::from_rgb(120, 165, 90)
}
fn text_primary() -> Color32 {
    Color32::from_rgb(240, 240, 240)
}
fn text_secondary() -> Color32 {
    Color32::from_rgb(170, 170, 170)
}
fn board_light() -> Color32 {
    Color32::from_rgb(238, 214, 176)
}
fn board_dark() -> Color32 {
    Color32::from_rgb(180, 136, 102)
}
fn board_border() -> Color32 {
    Color32::from_rgb(90, 60, 40)
}
fn highlight_selected() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 100, 120)
}
fn highlight_move() -> Color32 {
    Color32::from_rgba_unmultiplied(0, 0, 0, 70)
}
fn highlight_last() -> Color32 {
    Color32::from_rgba_unmultiplied(170, 210, 80, 100)
}
fn highlight_check() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 50, 50, 150)
}

/// Animation duration in seconds
const ANIMATION_DURATION: f32 = 0.15;

/// Piece animation state
#[derive(Clone)]
struct PieceAnimation {
    piece: Piece,
    from_rank: usize,
    from_file: usize,
    to_rank: usize,
    to_file: usize,
    start_time: Instant,
    duration: f32,
}

impl PieceAnimation {
    fn new(piece: Piece, from: (usize, usize), to: (usize, usize)) -> Self {
        PieceAnimation {
            piece,
            from_rank: from.0,
            from_file: from.1,
            to_rank: to.0,
            to_file: to.1,
            start_time: Instant::now(),
            duration: ANIMATION_DURATION,
        }
    }

    fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        (elapsed / self.duration).min(1.0)
    }

    fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Ease-out cubic for smooth deceleration
    fn eased_progress(&self) -> f32 {
        let t = self.progress();
        1.0 - (1.0 - t).powi(3)
    }

    fn current_position(&self, square_size: f32, board_min: Pos2, flipped: bool) -> Pos2 {
        let t = self.eased_progress();

        let from_screen = Self::board_to_screen(self.from_rank, self.from_file, flipped);
        let to_screen = Self::board_to_screen(self.to_rank, self.to_file, flipped);

        let x = from_screen.0 as f32 + (to_screen.0 as f32 - from_screen.0 as f32) * t;
        let y = from_screen.1 as f32 + (to_screen.1 as f32 - from_screen.1 as f32) * t;

        Pos2::new(
            board_min.x + (x + 0.5) * square_size,
            board_min.y + (y + 0.5) * square_size,
        )
    }

    fn board_to_screen(rank: usize, file: usize, flipped: bool) -> (usize, usize) {
        let screen_row = if flipped { rank } else { 7 - rank };
        let screen_col = if flipped { 7 - file } else { file };
        (screen_col, screen_row)
    }
}

/// Main chess application
pub struct ChessApp {
    board: Board,
    game_state: GameState,
    game_mode: GameMode,
    time_control: TimeControl,
    timer: GameTimer,
    bot: ChessBot,
    bot_difficulty: BotDifficulty,
    player_color: Color,

    // UI state
    selected_square: Option<(usize, usize)>,
    legal_moves: Vec<Move>,
    last_move: Option<Move>,
    promotion_pending: Option<(Move, Vec<PieceType>)>,

    // Board display
    flipped: bool,

    // Animation
    current_animation: Option<PieceAnimation>,
    pending_bot_move: bool,
}

impl Default for ChessApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessApp {
    pub fn new() -> Self {
        ChessApp {
            board: Board::new(),
            game_state: GameState::Title,
            game_mode: GameMode::VsBot,
            time_control: TimeControl::Normal,
            timer: GameTimer::new(TimeControl::Normal),
            bot: ChessBot::new(BotDifficulty::Medium),
            bot_difficulty: BotDifficulty::Medium,
            player_color: Color::White,
            selected_square: None,
            legal_moves: Vec::new(),
            last_move: None,
            promotion_pending: None,
            flipped: false,
            current_animation: None,
            pending_bot_move: false,
        }
    }

    fn start_game(&mut self) {
        self.board = Board::new();
        self.timer = GameTimer::new(self.time_control);
        self.bot = ChessBot::new(self.bot_difficulty);
        self.selected_square = None;
        self.legal_moves = Vec::new();
        self.last_move = None;
        self.promotion_pending = None;
        self.game_state = GameState::Playing;
        self.flipped = self.player_color == Color::Black && self.game_mode == GameMode::VsBot;
        self.current_animation = None;
        self.pending_bot_move = false;

        self.timer.start_white();

        if self.game_mode == GameMode::VsBot && self.player_color == Color::Black {
            self.pending_bot_move = true;
        }
    }

    fn make_bot_move(&mut self) {
        if let Some(mv) = self.bot.find_best_move(&self.board) {
            self.start_move_animation(mv, true);
        }
    }

    fn start_move_animation(&mut self, mv: Move, is_bot_move: bool) {
        // Get the piece being moved
        if let Some(piece) = self.board.get(mv.from_rank, mv.from_file) {
            self.current_animation = Some(PieceAnimation::new(
                piece,
                (mv.from_rank, mv.from_file),
                (mv.to_rank, mv.to_file),
            ));
            self.last_move = Some(mv);
            self.selected_square = None;
            self.legal_moves.clear();

            // If this is a player move and we're vs bot, schedule bot move after animation
            if !is_bot_move && self.game_mode == GameMode::VsBot {
                self.pending_bot_move = true;
            }
        }
    }

    fn complete_move_animation(&mut self) {
        if self.current_animation.is_some() {
            if let Some(mv) = self.last_move {
                self.board.make_move(mv);

                match self.board.current_turn {
                    Color::White => self.timer.start_white(),
                    Color::Black => self.timer.start_black(),
                }

                self.check_game_over();
            }
        }
        self.current_animation = None;
    }

    fn check_game_over(&mut self) {
        if MoveGenerator::is_checkmate(&self.board) {
            let result = match self.board.current_turn {
                Color::White => GameResult::BlackWins,
                Color::Black => GameResult::WhiteWins,
            };
            self.timer.stop_all();
            self.game_state = GameState::GameOver(result);
            return;
        }

        if MoveGenerator::is_stalemate(&self.board)
            || MoveGenerator::is_insufficient_material(&self.board)
            || MoveGenerator::is_fifty_move_draw(&self.board)
        {
            self.timer.stop_all();
            self.game_state = GameState::GameOver(GameResult::Draw);
            return;
        }

        if self.timer.white_flagged() {
            self.timer.stop_all();
            self.game_state = GameState::GameOver(GameResult::BlackWins);
        } else if self.timer.black_flagged() {
            self.timer.stop_all();
            self.game_state = GameState::GameOver(GameResult::WhiteWins);
        }
    }

    fn handle_square_click(&mut self, rank: usize, file: usize) {
        // Don't allow clicks during animation
        if self.current_animation.is_some() || self.promotion_pending.is_some() {
            return;
        }

        let can_move = match self.game_mode {
            GameMode::LocalPvP => true,
            GameMode::VsBot => self.board.current_turn == self.player_color,
        };

        if !can_move {
            return;
        }

        if let Some((sel_rank, sel_file)) = self.selected_square {
            if let Some(mv) = self.find_move(sel_rank, sel_file, rank, file) {
                if let Some(piece) = self.board.get(sel_rank, sel_file) {
                    if piece.piece_type == PieceType::Pawn {
                        let promotion_rank = piece.color.promotion_rank();
                        if rank == promotion_rank {
                            self.promotion_pending = Some((
                                Move::new(sel_rank, sel_file, rank, file),
                                vec![
                                    PieceType::Queen,
                                    PieceType::Rook,
                                    PieceType::Bishop,
                                    PieceType::Knight,
                                ],
                            ));
                            return;
                        }
                    }
                }

                // Start animated move
                self.start_move_animation(mv, false);
            } else {
                self.try_select_square(rank, file);
            }
        } else {
            self.try_select_square(rank, file);
        }
    }

    fn try_select_square(&mut self, rank: usize, file: usize) {
        if let Some(piece) = self.board.get(rank, file) {
            if piece.color == self.board.current_turn {
                self.selected_square = Some((rank, file));
                self.legal_moves = MoveGenerator::generate_legal_moves(&self.board)
                    .into_iter()
                    .filter(|mv| mv.from_rank == rank && mv.from_file == file)
                    .collect();
                return;
            }
        }
        self.selected_square = None;
        self.legal_moves.clear();
    }

    fn find_move(
        &self,
        from_rank: usize,
        from_file: usize,
        to_rank: usize,
        to_file: usize,
    ) -> Option<Move> {
        self.legal_moves
            .iter()
            .find(|mv| {
                mv.from_rank == from_rank
                    && mv.from_file == from_file
                    && mv.to_rank == to_rank
                    && mv.to_file == to_file
            })
            .copied()
    }

    fn handle_promotion_choice(&mut self, piece_type: PieceType) {
        if let Some((base_mv, _)) = self.promotion_pending.take() {
            let mv = base_mv.with_promotion(piece_type);
            // For promotions, animate the pawn moving to the promotion square
            self.start_move_animation(mv, false);
        }
    }

    fn draw_title_screen(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();

        ui.painter().rect_filled(
            Rect::from_min_size(ui.min_rect().min, available),
            0.0,
            bg_dark(),
        );

        ui.vertical_centered(|ui| {
            ui.add_space(available.y * 0.2);

            // Title
            ui.label(
                RichText::new("CHESS")
                    .size(72.0)
                    .color(text_primary())
                    .strong(),
            );

            ui.add_space(10.0);
            ui.label(RichText::new("Engine").size(28.0).color(text_secondary()));

            ui.add_space(available.y * 0.15);

            // Play button
            let play_btn = ui.add_sized(
                [220.0, 60.0],
                egui::Button::new(RichText::new("PLAY").size(28.0).color(text_primary()))
                    .fill(accent())
                    .stroke(Stroke::NONE)
                    .corner_radius(8.0),
            );

            if play_btn.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if play_btn.clicked() {
                self.game_state = GameState::ModeSelect;
            }

            ui.add_space(available.y * 0.2);

            ui.label(
                RichText::new("Built with Rust")
                    .size(14.0)
                    .color(text_secondary()),
            );
        });
    }

    fn draw_mode_select(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();

        ui.painter().rect_filled(
            Rect::from_min_size(ui.min_rect().min, available),
            0.0,
            bg_dark(),
        );

        // Collect button clicks
        let mut new_game_mode: Option<GameMode> = None;
        let mut new_time_control: Option<TimeControl> = None;
        let mut new_difficulty: Option<BotDifficulty> = None;
        let mut new_player_color: Option<Color> = None;
        let mut start_clicked = false;
        let mut back_clicked = false;

        let current_game_mode = self.game_mode;
        let current_time_control = self.time_control;
        let current_difficulty = self.bot_difficulty;
        let current_player_color = self.player_color;

        ui.vertical_centered(|ui| {
            ui.add_space(30.0);

            ui.label(
                RichText::new("Game Setup")
                    .size(36.0)
                    .color(text_primary())
                    .strong(),
            );

            ui.add_space(40.0);

            egui::Frame::new()
                .fill(bg_panel())
                .corner_radius(12.0)
                .inner_margin(30.0)
                .show(ui, |ui| {
                    ui.set_min_width(350.0);

                    // Game Mode
                    ui.label(
                        RichText::new("GAME MODE")
                            .size(14.0)
                            .color(text_secondary()),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if Self::option_button(
                            ui,
                            "vs Computer",
                            current_game_mode == GameMode::VsBot,
                        ) {
                            new_game_mode = Some(GameMode::VsBot);
                        }
                        ui.add_space(10.0);
                        if Self::option_button(
                            ui,
                            "Local 1v1",
                            current_game_mode == GameMode::LocalPvP,
                        ) {
                            new_game_mode = Some(GameMode::LocalPvP);
                        }
                    });

                    ui.add_space(25.0);

                    // Time Control
                    ui.label(
                        RichText::new("TIME CONTROL")
                            .size(14.0)
                            .color(text_secondary()),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if Self::option_button(
                            ui,
                            "Bullet 1m",
                            current_time_control == TimeControl::Bullet,
                        ) {
                            new_time_control = Some(TimeControl::Bullet);
                        }
                        ui.add_space(10.0);
                        if Self::option_button(
                            ui,
                            "Normal 10m",
                            current_time_control == TimeControl::Normal,
                        ) {
                            new_time_control = Some(TimeControl::Normal);
                        }
                    });

                    if current_game_mode == GameMode::VsBot {
                        ui.add_space(25.0);

                        // Difficulty
                        ui.label(
                            RichText::new("DIFFICULTY")
                                .size(14.0)
                                .color(text_secondary()),
                        );
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if Self::option_button(
                                ui,
                                "Easy",
                                current_difficulty == BotDifficulty::Easy,
                            ) {
                                new_difficulty = Some(BotDifficulty::Easy);
                            }
                            ui.add_space(10.0);
                            if Self::option_button(
                                ui,
                                "Medium",
                                current_difficulty == BotDifficulty::Medium,
                            ) {
                                new_difficulty = Some(BotDifficulty::Medium);
                            }
                            ui.add_space(10.0);
                            if Self::option_button(
                                ui,
                                "Hard",
                                current_difficulty == BotDifficulty::Hard,
                            ) {
                                new_difficulty = Some(BotDifficulty::Hard);
                            }
                        });

                        ui.add_space(25.0);

                        // Play as
                        ui.label(RichText::new("PLAY AS").size(14.0).color(text_secondary()));
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if Self::option_button(
                                ui,
                                "White",
                                current_player_color == Color::White,
                            ) {
                                new_player_color = Some(Color::White);
                            }
                            ui.add_space(10.0);
                            if Self::option_button(
                                ui,
                                "Black",
                                current_player_color == Color::Black,
                            ) {
                                new_player_color = Some(Color::Black);
                            }
                        });
                    }
                });

            ui.add_space(40.0);

            // Start button
            if ui
                .add_sized(
                    [200.0, 50.0],
                    egui::Button::new(RichText::new("START GAME").size(20.0).color(text_primary()))
                        .fill(accent())
                        .stroke(Stroke::NONE)
                        .corner_radius(8.0),
                )
                .clicked()
            {
                start_clicked = true;
            }

            ui.add_space(15.0);

            // Back button
            if ui
                .add_sized(
                    [120.0, 35.0],
                    egui::Button::new(RichText::new("Back").size(16.0).color(text_secondary()))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, text_secondary()))
                        .corner_radius(6.0),
                )
                .clicked()
            {
                back_clicked = true;
            }
        });

        // Apply changes after UI rendering
        if let Some(mode) = new_game_mode {
            self.game_mode = mode;
        }
        if let Some(tc) = new_time_control {
            self.time_control = tc;
        }
        if let Some(diff) = new_difficulty {
            self.bot_difficulty = diff;
        }
        if let Some(color) = new_player_color {
            self.player_color = color;
        }
        if start_clicked {
            self.start_game();
        }
        if back_clicked {
            self.game_state = GameState::Title;
        }
    }

    fn option_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
        let fill = if selected { accent() } else { bg_dark() };
        let text_color = if selected {
            text_primary()
        } else {
            text_secondary()
        };

        ui.add_sized(
            [100.0, 36.0],
            egui::Button::new(RichText::new(text).size(14.0).color(text_color))
                .fill(fill)
                .stroke(Stroke::new(
                    1.0,
                    if selected { accent() } else { text_secondary() },
                ))
                .corner_radius(6.0),
        )
        .clicked()
    }

    fn draw_piece_at(painter: &egui::Painter, piece: Piece, pos: Pos2, square_size: f32) {
        let symbol = piece.symbol();
        let text_color = match piece.color {
            Color::White => Color32::WHITE,
            Color::Black => Color32::from_rgb(30, 30, 30),
        };

        // Shadow for white pieces
        if piece.color == Color::White {
            painter.text(
                pos + Vec2::new(1.5, 1.5),
                egui::Align2::CENTER_CENTER,
                symbol,
                FontId::proportional(square_size * 0.78),
                Color32::from_rgba_unmultiplied(0, 0, 0, 80),
            );
        }

        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            symbol,
            FontId::proportional(square_size * 0.78),
            text_color,
        );
    }

    fn draw_game(&mut self, ui: &mut egui::Ui) {
        self.timer.update();

        // Handle animation completion
        let animation_complete = self
            .current_animation
            .as_ref()
            .is_some_and(|a| a.is_complete());
        if animation_complete {
            self.complete_move_animation();

            // If bot move is pending, trigger it after a short delay
            if self.pending_bot_move && self.game_state == GameState::Playing {
                self.pending_bot_move = false;
                self.make_bot_move();
            }
        }

        self.check_game_over();

        let available = ui.available_size();

        // Background
        ui.painter().rect_filled(
            Rect::from_min_size(ui.min_rect().min, available),
            0.0,
            bg_dark(),
        );

        // Calculate board size
        let board_size = (available.x - 250.0)
            .min(available.y - 40.0)
            .clamp(320.0, 560.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);

            // Left panel
            ui.vertical(|ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(180.0);
                ui.add_space(20.0);

                // Top player (opponent when not flipped)
                let (top_color, top_time) = if self.flipped {
                    (Color::White, self.timer.white.remaining())
                } else {
                    (Color::Black, self.timer.black.remaining())
                };
                self.draw_player_panel(
                    ui,
                    top_color,
                    top_time,
                    top_color == self.board.current_turn,
                );

                ui.add_space(20.0);

                // Game info
                egui::Frame::new()
                    .fill(bg_panel())
                    .corner_radius(8.0)
                    .inner_margin(15.0)
                    .show(ui, |ui| {
                        ui.set_min_width(150.0);

                        let turn_text = match self.board.current_turn {
                            Color::White => "White to move",
                            Color::Black => "Black to move",
                        };
                        ui.label(RichText::new(turn_text).size(14.0).color(text_primary()));

                        if self.board.is_in_check(self.board.current_turn) {
                            ui.add_space(5.0);
                            ui.label(
                                RichText::new("CHECK!")
                                    .size(16.0)
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .strong(),
                            );
                        }

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Move {}", self.board.fullmove_number))
                                .size(12.0)
                                .color(text_secondary()),
                        );
                    });

                ui.add_space(20.0);

                // Bottom player
                let (bottom_color, bottom_time) = if self.flipped {
                    (Color::Black, self.timer.black.remaining())
                } else {
                    (Color::White, self.timer.white.remaining())
                };
                self.draw_player_panel(
                    ui,
                    bottom_color,
                    bottom_time,
                    bottom_color == self.board.current_turn,
                );

                ui.add_space(30.0);

                // Buttons
                if ui
                    .add_sized(
                        [150.0, 32.0],
                        egui::Button::new(
                            RichText::new("Flip Board").size(13.0).color(text_primary()),
                        )
                        .fill(bg_panel())
                        .corner_radius(6.0),
                    )
                    .clicked()
                {
                    self.flipped = !self.flipped;
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(
                        [150.0, 32.0],
                        egui::Button::new(RichText::new("Resign").size(13.0).color(text_primary()))
                            .fill(Color32::from_rgb(140, 60, 60))
                            .corner_radius(6.0),
                    )
                    .clicked()
                {
                    let result = match self.board.current_turn {
                        Color::White => GameResult::BlackWins,
                        Color::Black => GameResult::WhiteWins,
                    };
                    self.timer.stop_all();
                    self.game_state = GameState::GameOver(result);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(
                        [150.0, 32.0],
                        egui::Button::new(
                            RichText::new("New Game").size(13.0).color(text_secondary()),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, text_secondary()))
                        .corner_radius(6.0),
                    )
                    .clicked()
                {
                    self.game_state = GameState::ModeSelect;
                }
            });

            ui.add_space(30.0);

            // Chess board
            ui.vertical(|ui| {
                ui.add_space(20.0);
                self.draw_board(ui, board_size);
            });
        });

        ui.ctx().request_repaint();
    }

    fn draw_player_panel(
        &self,
        ui: &mut egui::Ui,
        color: Color,
        time: std::time::Duration,
        is_turn: bool,
    ) {
        let border_color = if is_turn { accent() } else { bg_panel() };

        egui::Frame::new()
            .fill(bg_panel())
            .corner_radius(8.0)
            .stroke(Stroke::new(2.0, border_color))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_min_width(150.0);

                let color_name = match color {
                    Color::White => "White",
                    Color::Black => "Black",
                };

                ui.horizontal(|ui| {
                    // Color indicator
                    let indicator_color = match color {
                        Color::White => Color32::WHITE,
                        Color::Black => Color32::from_rgb(40, 40, 40),
                    };
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 8.0, indicator_color);
                    ui.painter().circle_stroke(
                        rect.center(),
                        8.0,
                        Stroke::new(1.0, text_secondary()),
                    );

                    ui.add_space(8.0);
                    ui.label(RichText::new(color_name).size(14.0).color(text_primary()));
                });

                ui.add_space(8.0);

                // Timer
                let time_color = if time.as_secs() < 30 {
                    Color32::from_rgb(255, 100, 100)
                } else {
                    text_primary()
                };

                ui.label(
                    RichText::new(format_time(time))
                        .size(28.0)
                        .color(time_color)
                        .monospace()
                        .strong(),
                );
            });
    }

    fn draw_board(&mut self, ui: &mut egui::Ui, board_size: f32) {
        let square_size = board_size / 8.0;
        let border_width = 8.0;
        let total_size = board_size + border_width * 2.0;

        let (response, painter) = ui.allocate_painter(Vec2::splat(total_size), Sense::click());
        let outer_rect = response.rect;

        // Board border
        painter.rect_filled(outer_rect, 4.0, board_border());

        let board_rect = Rect::from_min_size(
            outer_rect.min + Vec2::splat(border_width),
            Vec2::splat(board_size),
        );

        // Draw squares
        for rank in 0..8 {
            for file in 0..8 {
                let screen_row = if self.flipped { rank } else { 7 - rank };
                let screen_col = if self.flipped { 7 - file } else { file };

                let square_rect = Rect::from_min_size(
                    Pos2::new(
                        board_rect.min.x + screen_col as f32 * square_size,
                        board_rect.min.y + screen_row as f32 * square_size,
                    ),
                    Vec2::splat(square_size),
                );

                // Base square color
                let base_color = if (rank + file) % 2 == 0 {
                    board_dark()
                } else {
                    board_light()
                };
                painter.rect_filled(square_rect, 0.0, base_color);

                // Last move highlight
                if let Some(mv) = self.last_move {
                    if (rank == mv.from_rank && file == mv.from_file)
                        || (rank == mv.to_rank && file == mv.to_file)
                    {
                        painter.rect_filled(square_rect, 0.0, highlight_last());
                    }
                }

                // Check highlight
                if let Some(piece) = self.board.get(rank, file) {
                    if piece.piece_type == PieceType::King
                        && piece.color == self.board.current_turn
                        && self.board.is_in_check(self.board.current_turn)
                    {
                        painter.rect_filled(square_rect, 0.0, highlight_check());
                    }
                }

                // Selected square highlight
                if let Some((sel_rank, sel_file)) = self.selected_square {
                    if rank == sel_rank && file == sel_file {
                        painter.rect_filled(square_rect, 0.0, highlight_selected());
                    }
                }

                // Legal move hints
                for mv in &self.legal_moves {
                    if mv.to_rank == rank && mv.to_file == file {
                        let center = square_rect.center();
                        if self.board.get(rank, file).is_some() {
                            // Capture: ring around edge
                            painter.circle_stroke(
                                center,
                                square_size / 2.0 - 4.0,
                                Stroke::new(4.0, highlight_move()),
                            );
                        } else {
                            // Move: dot in center
                            painter.circle_filled(center, square_size / 5.0, highlight_move());
                        }
                    }
                }

                // Draw piece (skip if it's the animating piece at source)
                let is_animating_source = self
                    .current_animation
                    .as_ref()
                    .is_some_and(|anim| anim.from_rank == rank && anim.from_file == file);

                if !is_animating_source {
                    if let Some(piece) = self.board.get(rank, file) {
                        Self::draw_piece_at(&painter, piece, square_rect.center(), square_size);
                    }
                }

                // Coordinates
                if screen_row == 7 {
                    let label = (b'a' + file as u8) as char;
                    let label_color = if (rank + file) % 2 == 0 {
                        board_light()
                    } else {
                        board_dark()
                    };
                    painter.text(
                        Pos2::new(square_rect.right() - 4.0, square_rect.bottom() - 4.0),
                        egui::Align2::RIGHT_BOTTOM,
                        label,
                        FontId::proportional(11.0),
                        label_color,
                    );
                }
                if screen_col == 0 {
                    let label = (b'1' + rank as u8) as char;
                    let label_color = if (rank + file) % 2 == 0 {
                        board_light()
                    } else {
                        board_dark()
                    };
                    painter.text(
                        Pos2::new(square_rect.left() + 4.0, square_rect.top() + 2.0),
                        egui::Align2::LEFT_TOP,
                        label,
                        FontId::proportional(11.0),
                        label_color,
                    );
                }
            }
        }

        // Draw animating piece on top
        if let Some(ref anim) = self.current_animation {
            let pos = anim.current_position(square_size, board_rect.min, self.flipped);
            Self::draw_piece_at(&painter, anim.piece, pos, square_size);
        }

        // Handle clicks
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel_pos = pos - board_rect.min;
                let screen_col = (rel_pos.x / square_size) as usize;
                let screen_row = (rel_pos.y / square_size) as usize;

                if screen_col < 8 && screen_row < 8 {
                    let file = if self.flipped {
                        7 - screen_col
                    } else {
                        screen_col
                    };
                    let rank = if self.flipped {
                        screen_row
                    } else {
                        7 - screen_row
                    };

                    self.handle_square_click(rank, file);
                }
            }
        }

        // Promotion dialog
        if let Some((_, ref pieces)) = self.promotion_pending {
            let dialog_width = 4.0 * square_size + 20.0;
            let dialog_height = square_size + 30.0;
            let dialog_rect =
                Rect::from_center_size(outer_rect.center(), Vec2::new(dialog_width, dialog_height));

            painter.rect_filled(dialog_rect, 8.0, bg_panel());
            painter.rect_stroke(
                dialog_rect,
                8.0,
                Stroke::new(2.0, accent()),
                egui::StrokeKind::Outside,
            );

            let promo_color = self.board.current_turn;

            for (i, &piece_type) in pieces.iter().enumerate() {
                let piece = crate::pieces::Piece::new(promo_color, piece_type);
                let piece_rect = Rect::from_min_size(
                    Pos2::new(
                        dialog_rect.min.x + 10.0 + i as f32 * square_size,
                        dialog_rect.min.y + 15.0,
                    ),
                    Vec2::splat(square_size),
                );

                let text_color = match promo_color {
                    Color::White => Color32::WHITE,
                    Color::Black => Color32::from_rgb(30, 30, 30),
                };

                painter.text(
                    piece_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    piece.symbol(),
                    FontId::proportional(square_size * 0.75),
                    text_color,
                );
            }
        }

        // Handle promotion clicks
        if self.promotion_pending.is_some() && response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let dialog_width = 4.0 * square_size + 20.0;
                let dialog_rect = Rect::from_center_size(
                    outer_rect.center(),
                    Vec2::new(dialog_width, square_size + 30.0),
                );

                if dialog_rect.contains(pos) {
                    let rel_x = pos.x - dialog_rect.min.x - 10.0;
                    let index = (rel_x / square_size) as usize;
                    if index < 4 {
                        let pieces = [
                            PieceType::Queen,
                            PieceType::Rook,
                            PieceType::Bishop,
                            PieceType::Knight,
                        ];
                        self.handle_promotion_choice(pieces[index]);
                    }
                }
            }
        }
    }

    fn draw_game_over(&mut self, ui: &mut egui::Ui) {
        if let GameState::GameOver(result) = self.game_state {
            let available = ui.available_size();

            // Background
            ui.painter().rect_filled(
                Rect::from_min_size(ui.min_rect().min, available),
                0.0,
                bg_dark(),
            );

            let board_size = available.x.min(available.y - 100.0).clamp(280.0, 450.0);

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                self.draw_board_static(ui, board_size);
            });

            // Overlay window
            egui::Window::new("Game Over")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::new()
                        .fill(bg_panel())
                        .corner_radius(12.0)
                        .inner_margin(30.0)
                        .stroke(Stroke::new(2.0, accent())),
                )
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        let result_text = match result {
                            GameResult::WhiteWins => "White Wins!",
                            GameResult::BlackWins => "Black Wins!",
                            GameResult::Draw => "Draw!",
                        };

                        ui.label(
                            RichText::new(result_text)
                                .size(32.0)
                                .color(text_primary())
                                .strong(),
                        );

                        ui.add_space(15.0);

                        let reason = if MoveGenerator::is_checkmate(&self.board) {
                            "Checkmate"
                        } else if MoveGenerator::is_stalemate(&self.board) {
                            "Stalemate"
                        } else if self.timer.white_flagged() {
                            "White flagged"
                        } else if self.timer.black_flagged() {
                            "Black flagged"
                        } else if MoveGenerator::is_insufficient_material(&self.board) {
                            "Insufficient material"
                        } else if MoveGenerator::is_fifty_move_draw(&self.board) {
                            "50-move rule"
                        } else {
                            "Resignation"
                        };

                        ui.label(RichText::new(reason).size(16.0).color(text_secondary()));

                        ui.add_space(25.0);

                        if ui
                            .add_sized(
                                [180.0, 45.0],
                                egui::Button::new(
                                    RichText::new("Play Again").size(18.0).color(text_primary()),
                                )
                                .fill(accent())
                                .corner_radius(8.0),
                            )
                            .clicked()
                        {
                            self.game_state = GameState::ModeSelect;
                        }

                        ui.add_space(10.0);

                        if ui
                            .add_sized(
                                [120.0, 32.0],
                                egui::Button::new(
                                    RichText::new("Main Menu")
                                        .size(14.0)
                                        .color(text_secondary()),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, text_secondary()))
                                .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            self.game_state = GameState::Title;
                        }
                    });
                });
        }
    }

    fn draw_board_static(&self, ui: &mut egui::Ui, board_size: f32) {
        let square_size = board_size / 8.0;
        let border_width = 6.0;
        let total_size = board_size + border_width * 2.0;

        let (_, painter) = ui.allocate_painter(Vec2::splat(total_size), Sense::hover());
        let outer_rect = Rect::from_min_size(ui.min_rect().min, Vec2::splat(total_size));

        painter.rect_filled(outer_rect, 4.0, board_border());

        let board_rect = Rect::from_min_size(
            outer_rect.min + Vec2::splat(border_width),
            Vec2::splat(board_size),
        );

        for rank in 0..8 {
            for file in 0..8 {
                let screen_row = if self.flipped { rank } else { 7 - rank };
                let screen_col = if self.flipped { 7 - file } else { file };

                let square_rect = Rect::from_min_size(
                    Pos2::new(
                        board_rect.min.x + screen_col as f32 * square_size,
                        board_rect.min.y + screen_row as f32 * square_size,
                    ),
                    Vec2::splat(square_size),
                );

                let base_color = if (rank + file) % 2 == 0 {
                    board_dark()
                } else {
                    board_light()
                };
                painter.rect_filled(square_rect, 0.0, base_color);

                if let Some(mv) = self.last_move {
                    if (rank == mv.from_rank && file == mv.from_file)
                        || (rank == mv.to_rank && file == mv.to_file)
                    {
                        painter.rect_filled(square_rect, 0.0, highlight_last());
                    }
                }

                if let Some(piece) = self.board.get(rank, file) {
                    let symbol = piece.symbol();
                    let text_color = match piece.color {
                        Color::White => Color32::WHITE,
                        Color::Black => Color32::from_rgb(30, 30, 30),
                    };

                    if piece.color == Color::White {
                        painter.text(
                            square_rect.center() + Vec2::new(1.5, 1.5),
                            egui::Align2::CENTER_CENTER,
                            symbol,
                            FontId::proportional(square_size * 0.78),
                            Color32::from_rgba_unmultiplied(0, 0, 0, 80),
                        );
                    }

                    painter.text(
                        square_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        symbol,
                        FontId::proportional(square_size * 0.78),
                        text_color,
                    );
                }
            }
        }
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg_dark()))
            .show(ctx, |ui| match self.game_state {
                GameState::Title => self.draw_title_screen(ui),
                GameState::ModeSelect => self.draw_mode_select(ui),
                GameState::Playing => self.draw_game(ui),
                GameState::GameOver(_) => self.draw_game_over(ui),
            });
    }
}

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 550.0])
            .with_title("Chess"),
        ..Default::default()
    };

    eframe::run_native(
        "Chess",
        options,
        Box::new(|_cc| Ok(Box::new(ChessApp::new()))),
    )
}
