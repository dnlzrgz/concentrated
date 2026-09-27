use std::time::Duration;

use crate::message::Message;

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
    fn duration(&self) -> Duration {
        match self {
            SessionType::Work => Duration::from_secs(25 * 60),
            SessionType::Break => Duration::from_secs(5 * 60),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Break => "break",
        }
    }
}

#[derive(Debug, Default)]
pub struct Model {
    state: State,
    session_type: SessionType,
    remaining: Duration,
    should_quit: bool,
}

impl Model {
    pub fn new() -> Self {
        let session_type = SessionType::default();
        let remaining = session_type.duration();
        Self {
            remaining,
            session_type,
            ..Default::default()
        }
    }
}

impl Model {
    pub fn update(&mut self, msg: Message) -> color_eyre::Result<()> {
        match msg {
            Message::Tick => {
                if self.state == State::Running {
                    self.remaining = self.remaining.saturating_sub(Duration::from_secs(1));
                    if self.remaining.is_zero() {
                        self.skip_session();
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
                self.remaining = self.session_type.duration();
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

        self.remaining = self.session_type.duration()
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

    pub fn total(&self) -> Duration {
        self.session_type.duration()
    }

    pub fn session_label(&self) -> &'static str {
        self.session_type.label()
    }
}
