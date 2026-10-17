use std::time::Duration;

use crate::{cli::Cli, watch_face::WatchFace};

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub work_duration: Duration,
    pub break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_before_long_break: u32,
    pub notifications: bool,
    pub auto_start: bool,
    pub watch_face: WatchFace,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_before_long_break: 4,
            notifications: true,
            auto_start: false,
            watch_face: WatchFace::Large,
        }
    }
}

impl Config {
    pub fn resolve(cli: &Cli) -> Self {
        let d = Self::default();
        Self {
            work_duration: cli.work.unwrap_or(d.work_duration),
            break_duration: cli.break_.unwrap_or(d.break_duration),
            long_break_duration: cli.long_break.unwrap_or(d.long_break_duration),
            sessions_before_long_break: cli.sessions.unwrap_or(d.sessions_before_long_break),
            notifications: cli.notify().unwrap_or(d.notifications),
            auto_start: cli.auto_start().unwrap_or(d.auto_start),
            watch_face: cli.face.unwrap_or(d.watch_face),
        }
    }
}
