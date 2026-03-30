use std::time::{Duration, Instant};

/// Time control mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeControl {
    Bullet,  // 1 minute per side
    Normal,  // 10 minutes per side
    Custom(Duration),
}

impl TimeControl {
    pub fn duration(self) -> Duration {
        match self {
            TimeControl::Bullet => Duration::from_secs(60),
            TimeControl::Normal => Duration::from_secs(600),
            TimeControl::Custom(d) => d,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TimeControl::Bullet => "Bullet (1 min)",
            TimeControl::Normal => "Normal (10 min)",
            TimeControl::Custom(_) => "Custom",
        }
    }
}

/// Chess timer for one player
#[derive(Debug, Clone)]
pub struct PlayerTimer {
    remaining: Duration,
    running: bool,
    last_tick: Option<Instant>,
}

impl PlayerTimer {
    pub fn new(initial: Duration) -> Self {
        PlayerTimer {
            remaining: initial,
            running: false,
            last_tick: None,
        }
    }

    pub fn start(&mut self) {
        if !self.running {
            self.running = true;
            self.last_tick = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if self.running {
            self.update();
            self.running = false;
            self.last_tick = None;
        }
    }

    pub fn update(&mut self) {
        if self.running {
            if let Some(last) = self.last_tick {
                let elapsed = last.elapsed();
                self.remaining = self.remaining.saturating_sub(elapsed);
                self.last_tick = Some(Instant::now());
            }
        }
    }

    pub fn remaining(&self) -> Duration {
        if self.running {
            if let Some(last) = self.last_tick {
                return self.remaining.saturating_sub(last.elapsed());
            }
        }
        self.remaining
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_expired(&self) -> bool {
        self.remaining() == Duration::ZERO
    }

    pub fn reset(&mut self, initial: Duration) {
        self.remaining = initial;
        self.running = false;
        self.last_tick = None;
    }
}

/// Game timer managing both players
#[derive(Debug, Clone)]
pub struct GameTimer {
    pub white: PlayerTimer,
    pub black: PlayerTimer,
    time_control: TimeControl,
}

impl GameTimer {
    pub fn new(time_control: TimeControl) -> Self {
        let duration = time_control.duration();
        GameTimer {
            white: PlayerTimer::new(duration),
            black: PlayerTimer::new(duration),
            time_control,
        }
    }

    pub fn start_white(&mut self) {
        self.black.stop();
        self.white.start();
    }

    pub fn start_black(&mut self) {
        self.white.stop();
        self.black.start();
    }

    pub fn stop_all(&mut self) {
        self.white.stop();
        self.black.stop();
    }

    pub fn update(&mut self) {
        self.white.update();
        self.black.update();
    }

    pub fn reset(&mut self) {
        let duration = self.time_control.duration();
        self.white.reset(duration);
        self.black.reset(duration);
    }

    pub fn time_control(&self) -> TimeControl {
        self.time_control
    }

    pub fn set_time_control(&mut self, tc: TimeControl) {
        self.time_control = tc;
        self.reset();
    }

    pub fn white_flagged(&self) -> bool {
        self.white.is_expired()
    }

    pub fn black_flagged(&self) -> bool {
        self.black.is_expired()
    }
}

/// Format duration as MM:SS
pub fn format_time(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    let tenths = duration.subsec_millis() / 100;

    if total_secs < 10 {
        format!("{}:{:02}.{}", mins, secs, tenths)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}
