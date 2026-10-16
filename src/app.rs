use std::time::Duration;

use crate::{config::Config, message::Message};

#[derive(Debug, Default, PartialEq, Eq)]
enum State {
    Running,
    #[default]
    Stopped,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum SessionType {
    #[default]
    Work,
    Break,
    LongBreak,
}

impl SessionType {
    fn duration(&self, cfg: &Config) -> Duration {
        match self {
            SessionType::Work => cfg.work_duration,
            SessionType::Break => cfg.break_duration,
            SessionType::LongBreak => cfg.long_break_duration,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Break => "break",
            Self::LongBreak => "long break",
        }
    }
}

#[derive(Debug)]
pub struct Model {
    config: Config,
    state: State,
    session_type: SessionType,
    completed_work_sessions: u32,
    remaining: Duration,
    should_quit: bool,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let session_type = SessionType::default();
        Self {
            remaining: session_type.duration(&config),
            config,
            session_type,
            completed_work_sessions: 0,
            state: if config.auto_start {
                State::Running
            } else {
                State::Stopped
            },
            should_quit: false,
        }
    }
}

impl Model {
    fn session_duration(&self) -> Duration {
        self.session_type.duration(&self.config)
    }

    pub fn update(&mut self, msg: Message) -> color_eyre::Result<()> {
        match msg {
            Message::Tick => {
                if self.state == State::Running {
                    self.remaining = self.remaining.saturating_sub(Duration::from_secs(1));
                    if self.remaining.is_zero() {
                        self.finish_session();
                    }
                }
            }
            Message::ToggleStartStop => {
                self.state = match self.state {
                    State::Running => State::Stopped,
                    State::Stopped => State::Running,
                };
            }
            Message::ToggleAutoStart => self.toggle_auto_start(),
            Message::Reset => {
                self.state = State::Stopped;
                self.remaining = self.session_duration();
            }
            Message::Skip => self.advance_session(),
            Message::Quit => self.should_quit = true,
        }

        Ok(())
    }

    /// Toggle auto-start. Enabling it while stopped starts the current session.
    fn toggle_auto_start(&mut self) {
        self.config.auto_start = !self.config.auto_start;
        if self.config.auto_start && self.state == State::Stopped {
            self.state = State::Running;
        }
    }

    /// Moves to the next session. Used when a session runs out and when its skipped.
    fn advance_session(&mut self) {
        match self.session_type {
            SessionType::Work => {
                self.completed_work_sessions += 1;
                let sessions = self.config.sessions_before_long_break;
                self.session_type =
                    if sessions > 0 && self.completed_work_sessions.is_multiple_of(sessions) {
                        SessionType::LongBreak
                    } else {
                        SessionType::Break
                    };
            }
            SessionType::Break => self.session_type = SessionType::Work,
            SessionType::LongBreak => {
                self.completed_work_sessions = 0;
                self.session_type = SessionType::Work;
            }
        }

        self.remaining = self.session_duration()
    }

    fn finish_session(&mut self) {
        self.advance_session();
        if !self.config.auto_start {
            self.state = State::Stopped;
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn is_running(&self) -> bool {
        self.state == State::Running
    }

    pub fn auto_start(&self) -> bool {
        self.config.auto_start
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn session_label(&self) -> &'static str {
        self.session_type.label()
    }

    /// Tracks current session within the cycle.
    pub fn session_progress(&self) -> Option<(u32, u32)> {
        if self.config.sessions_before_long_break == 0 {
            return None;
        }

        let current = match self.session_type {
            SessionType::Work => self.completed_work_sessions.saturating_add(1),
            SessionType::Break => self.completed_work_sessions,
            SessionType::LongBreak => return None,
        };

        Some((current, self.config.sessions_before_long_break))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_with_sessions(sessions_before_long_break: u32) -> Model {
        Model::new(Config {
            sessions_before_long_break,
            ..Config::default()
        })
    }

    fn skip(model: &mut Model) {
        model.update(Message::Skip).unwrap();
    }

    #[test]
    fn long_break_follows_the_last_work_session() {
        let mut model = model_with_sessions(4);

        for completed in 1..=3 {
            assert_eq!(model.session_type, SessionType::Work);
            skip(&mut model);
            assert_eq!(model.session_type, SessionType::Break);
            assert_eq!(model.completed_work_sessions, completed);
            skip(&mut model);
        }

        assert_eq!(model.session_type, SessionType::Work);
        skip(&mut model);
        assert_eq!(model.session_type, SessionType::LongBreak);
        assert_eq!(model.completed_work_sessions, 4);
    }

    #[test]
    fn long_break_resets_the_cycle() {
        let mut model = model_with_sessions(2);

        skip(&mut model); // work 1 -> break
        skip(&mut model); // break -> work 2
        skip(&mut model); // work 2 -> long break
        assert_eq!(model.session_type, SessionType::LongBreak);

        skip(&mut model); // long break -> work 1 of the next cycle
        assert_eq!(model.session_type, SessionType::Work);
        assert_eq!(model.completed_work_sessions, 0);
        assert_eq!(model.session_progress(), Some((1, 2)));
    }

    #[test]
    fn zero_sessions_disables_long_breaks() {
        let mut model = model_with_sessions(0);

        for _ in 0..10 {
            assert_eq!(model.session_type, SessionType::Work);
            skip(&mut model);
            assert_eq!(model.session_type, SessionType::Break);
            skip(&mut model);
        }

        assert_eq!(model.session_progress(), None);
    }

    #[test]
    fn session_progress_tracks_the_current_cycle() {
        let mut model = model_with_sessions(4);
        assert_eq!(model.session_progress(), Some((1, 4)));

        skip(&mut model); // finish work 1 -> break
        assert_eq!(model.session_progress(), Some((1, 4)));

        skip(&mut model); // break -> work 2
        assert_eq!(model.session_progress(), Some((2, 4)));

        skip(&mut model); // finish work 2 -> break
        assert_eq!(model.session_progress(), Some((2, 4)));
    }

    #[test]
    fn long_break_uses_the_configured_duration() {
        let long_break = Duration::from_secs(15 * 60);
        let mut model = Model::new(Config {
            long_break_duration: long_break,
            sessions_before_long_break: 1,
            ..Config::default()
        });

        skip(&mut model);
        assert_eq!(model.session_type, SessionType::LongBreak);
        assert_eq!(model.remaining(), long_break);
    }

    #[test]
    fn reset_preserves_cycle_progress() {
        let mut model = model_with_sessions(4);
        skip(&mut model); // work 1 -> break
        skip(&mut model); // break -> work 2

        model.update(Message::Reset).unwrap();

        assert_eq!(model.session_type, SessionType::Work);
        assert_eq!(model.session_progress(), Some((2, 4)));
        assert_eq!(model.remaining(), model.session_duration());
    }

    #[test]
    fn session_progress_is_hidden_during_long_break() {
        let mut model = model_with_sessions(1);
        assert_eq!(model.session_progress(), Some((1, 1)));

        skip(&mut model); // work 1 -> long break
        assert_eq!(model.session_type, SessionType::LongBreak);
        assert_eq!(model.session_progress(), None);
    }

    #[test]
    fn session_progress_stays_visible_during_short_break() {
        let mut model = model_with_sessions(2);

        skip(&mut model); // work 1 -> break
        assert_eq!(model.session_type, SessionType::Break);
        assert_eq!(model.session_progress(), Some((1, 2)));
    }

    #[test]
    fn enabling_auto_start_while_stopped_starts_the_session() {
        let mut model = model_with_sessions(4);
        assert!(!model.is_running());
        assert!(!model.auto_start());

        model.update(Message::ToggleAutoStart).unwrap();

        assert!(model.auto_start());
        assert!(model.is_running());
    }

    #[test]
    fn disabling_auto_start_while_running_keeps_it_running() {
        let mut model = Model::new(Config {
            auto_start: true,
            ..Config::default()
        });
        assert!(model.is_running());

        model.update(Message::ToggleAutoStart).unwrap();

        assert!(!model.auto_start());
        assert!(model.is_running());
    }

    #[test]
    fn toggling_auto_start_twice_restores_the_original_state() {
        let mut model = Model::new(Config {
            auto_start: true,
            ..Config::default()
        });
        assert!(model.auto_start());
        assert!(model.is_running());

        model.update(Message::ToggleAutoStart).unwrap();
        model.update(Message::ToggleAutoStart).unwrap();

        assert!(model.auto_start());
        assert!(model.is_running());
    }
}
