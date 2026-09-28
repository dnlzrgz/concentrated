use std::time::Duration;

use crate::cli::Cli;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub work_duration: Duration,
    pub break_duration: Duration,
    pub notifications: bool,
    pub auto_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            break_duration: Duration::from_secs(5 * 60),
            notifications: true,
            auto_start: false,
        }
    }
}

impl Config {
    pub fn resolve(cli: &Cli) -> Self {
        let d = Self::default();
        Self {
            work_duration: cli.work.unwrap_or(d.work_duration),
            break_duration: cli.break_.unwrap_or(d.break_duration),
            notifications: cli.notify().unwrap_or(d.notifications),
            auto_start: cli.auto_start().unwrap_or(d.auto_start),
        }
    }
}
