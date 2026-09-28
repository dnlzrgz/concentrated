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
}

impl SessionType {
    fn duration(&self, cfg: &Config) -> Duration {
        match self {
            SessionType::Work => cfg.work_duration,
            SessionType::Break => cfg.break_duration,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Break => "break",
        }
    }
}

#[derive(Debug)]
pub struct Model {
    config: Config,
    state: State,
    session_type: SessionType,
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
            Message::Reset => {
                self.state = State::Stopped;
                self.remaining = self.session_duration();
            }
            Message::Skip => self.skip_session(),
            Message::Quit => self.should_quit = true,
        }

        Ok(())
    }

    fn skip_session(&mut self) {
        self.session_type = match self.session_type {
            SessionType::Work => SessionType::Break,
            SessionType::Break => SessionType::Work,
        };

        self.remaining = self.session_duration()
    }

    fn finish_session(&mut self) {
        self.skip_session();
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

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn session_label(&self) -> &'static str {
        self.session_type.label()
    }
}
