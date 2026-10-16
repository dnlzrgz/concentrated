use std::{path::PathBuf, time::Duration};

use clap::Parser;

const MAX_SESSION: Duration = Duration::from_secs(59 * 60);

fn parse_session_duration(s: &str) -> Result<Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d.is_zero() {
        return Err("duration must be greater than zero".into());
    }

    if d > MAX_SESSION {
        return Err("duration cannot be greater than 59m".into());
    }

    Ok(d)
}

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to config file (overrides default location)
    #[arg(long, short)]
    pub config: Option<PathBuf>,

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

    /// Start the timer immediately and keep running between sessions
    #[arg(long, overrides_with = "no_auto_start")]
    auto_start: bool,

    /// Wait for manual start (default)
    #[arg(long, overrides_with = "auto_start")]
    no_auto_start: bool,

    /// Enable desktop notifications (on by default)
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
}
