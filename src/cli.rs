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

    /// Enable desktop notifications (on by default)
    #[arg(long, overrides_with = "no_notify")]
    notify: bool,

    /// Disable desktop notifications
    #[arg(long, overrides_with = "notify")]
    no_notify: bool,
}

impl Cli {
    pub fn notify(&self) -> Option<bool> {
        match (self.notify, self.no_notify) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        }
    }
}
