use std::{path::PathBuf, time::Duration};

use clap::Parser;

use crate::{
    config::{Settings, parse_session_duration},
    watch_face::WatchFace,
};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to config file (default: concentrated/config.toml)
    #[arg(long, short)]
    pub config: Option<PathBuf>,

    /// Profile to use
    #[arg(long, short)]
    pub profile: Option<String>,

    /// List the profiles and exit
    #[arg(long)]
    pub list_profiles: bool,

    /// Work session duration, e.g. "25m"
    #[arg(long, value_parser= parse_session_duration)]
    pub work: Option<Duration>,

    /// Break session duration, e.g. "5m"
    #[arg(long="break", value_parser= parse_session_duration)]
    pub break_: Option<Duration>,

    /// Long break session duration, e.g. "15m"
    #[arg(long, value_parser= parse_session_duration)]
    pub long_break: Option<Duration>,

    /// Work sessions before a long break, e.g. "4" (0 disables long breaks)
    #[arg(long)]
    pub sessions: Option<u32>,

    /// Watch face style
    #[arg(long, value_enum)]
    pub face: Option<WatchFace>,

    /// Start the timer immediately and keep running between sessions
    #[arg(long, overrides_with = "no_auto_start")]
    auto_start: bool,

    /// Wait for manual start (default)
    #[arg(long, overrides_with = "auto_start")]
    no_auto_start: bool,

    /// Enable desktop notifications (default)
    #[arg(long, overrides_with = "no_notify")]
    notify: bool,

    /// Disable desktop notifications
    #[arg(long, overrides_with = "notify")]
    no_notify: bool,
}

fn flag(on: bool, off: bool) -> Option<bool> {
    match (on, off) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    }
}

impl Cli {
    pub fn notify(&self) -> Option<bool> {
        flag(self.notify, self.no_notify)
    }

    pub fn auto_start(&self) -> Option<bool> {
        flag(self.auto_start, self.no_auto_start)
    }

    /// The flags the user passed, as the highest-priority settings layer.
    pub fn settings(&self) -> Settings {
        Settings {
            work: self.work,
            break_: self.break_,
            long_break: self.long_break,
            sessions: self.sessions,
            face: self.face,
            auto_start: self.auto_start(),
            notify: self.notify(),
            on_session_end: None, // config file only
        }
    }
}
