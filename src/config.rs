use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use etcetera::BaseStrategy;
use serde::{Deserialize, Deserializer, de::Error as _};

use crate::watch_face::WatchFace;

const MAX_SESSION: Duration = Duration::from_secs(59 * 60);
const DEFAULT_PROFILE: &str = "default";

/// Parses a session length like `25m`. Shared by CLI and config file.
pub fn parse_session_duration(s: &str) -> Result<Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;

    if d.is_zero() {
        return Err("duration must be greater than zero".into());
    }

    if d > MAX_SESSION {
        return Err("duration cannot be greater than 59m".into());
    }

    Ok(d)
}

fn deserialize_duration<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Duration>, D::Error> {
    Option::<String>::deserialize(d)?
        .map(|s| parse_session_duration(&s))
        .transpose()
        .map_err(D::Error::custom)
}

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
            watch_face: WatchFace::DigitalLarge,
        }
    }
}

impl Config {
    /// Layers, lowest to highest priority: built-in defaults, `[profiles.default]`,
    /// the selected profile, then `cli`.
    pub fn resolve(file: &FileConfig, profile: Option<&str>, cli: Settings) -> Result<Self> {
        let selected = file.selected_profile(profile)?;
        let base = file
            .profiles
            .get(DEFAULT_PROFILE)
            .copied()
            .unwrap_or_default();

        Ok(cli.or(selected).or(base).into_config())
    }
}

/// Partial configuration.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_duration")]
    pub work: Option<Duration>,
    #[serde(rename = "break", deserialize_with = "deserialize_duration")]
    pub break_: Option<Duration>,
    #[serde(deserialize_with = "deserialize_duration")]
    pub long_break: Option<Duration>,
    pub sessions: Option<u32>,
    pub face: Option<WatchFace>,
    pub auto_start: Option<bool>,
    pub notify: Option<bool>,
}

impl Settings {
    pub fn or(self, lower: Self) -> Self {
        Self {
            work: self.work.or(lower.work),
            break_: self.break_.or(lower.break_),
            long_break: self.long_break.or(lower.long_break),
            sessions: self.sessions.or(lower.sessions),
            face: self.face.or(lower.face),
            auto_start: self.auto_start.or(lower.auto_start),
            notify: self.notify.or(lower.notify),
        }
    }

    fn into_config(self) -> Config {
        let d = Config::default();
        Config {
            work_duration: self.work.unwrap_or(d.work_duration),
            break_duration: self.break_.unwrap_or(d.break_duration),
            long_break_duration: self.long_break.unwrap_or(d.long_break_duration),
            sessions_before_long_break: self.sessions.unwrap_or(d.sessions_before_long_break),
            notifications: self.notify.unwrap_or(d.notifications),
            auto_start: self.auto_start.unwrap_or(d.auto_start),
            watch_face: self.face.unwrap_or(d.watch_face),
        }
    }
}

/// On-disk config file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct FileConfig {
    default_profile: Option<String>,
    profiles: BTreeMap<String, Settings>,
}

impl FileConfig {
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Loads the config. Missing file is only fine at the default location.
    pub fn load(explicit: Option<&Path>) -> Result<Self> {
        let Some(path) = explicit.map(Path::to_path_buf).or_else(default_path) else {
            return Ok(Self::default());
        };

        match fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text)
                .wrap_err_with(|| format!("invalid config file {}", path.display())),
            Err(e) if e.kind() == io::ErrorKind::NotFound && explicit.is_none() => {
                Ok(Self::default())
            }
            Err(e) => {
                Err(e).wrap_err_with(|| format!("cannot read config file {}", path.display()))
            }
        }
    }

    /// Name of the profile that runs: `--profile`, else `default-profile`, else "default".
    fn active_name<'a>(&'a self, cli_profile: Option<&'a str>) -> &'a str {
        cli_profile
            .or(self.default_profile.as_deref())
            .unwrap_or(DEFAULT_PROFILE)
    }

    fn selected_profile(&self, cli_profile: Option<&str>) -> Result<Settings> {
        let name = self.active_name(cli_profile);
        if let Some(settings) = self.profiles.get(name) {
            return Ok(*settings);
        }

        if name == DEFAULT_PROFILE {
            return Ok(Settings::default());
        }

        let available = self
            .profile_names()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        if cli_profile.is_none() {
            bail!(
                "`default-profile` is set to \"{name}\", which is not defined (available: {available})"
            );
        }

        bail!("profile \"{name}\" not found (available: {available})")
    }

    /// Sorted list of profile names.
    fn profile_names(&self) -> BTreeSet<&str> {
        self.profiles
            .keys()
            .map(String::as_str)
            .chain([DEFAULT_PROFILE])
            .collect()
    }

    /// Profile per line, signaling the one that would run.
    pub fn list_profiles(&self, cli_profile: Option<&str>) -> String {
        let active = self.active_name(cli_profile);
        let mut out = String::new();
        for name in self.profile_names() {
            let mark = if name == active { '*' } else { ' ' };
            let _ = writeln!(out, "{mark} {name}");
        }

        out
    }
}

fn default_path() -> Option<PathBuf> {
    let strategy = etcetera::choose_base_strategy().ok()?;
    Some(
        strategy
            .config_dir()
            .join("concentrated")
            .join("config.toml"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../config.example.toml");

    fn mins(m: u64) -> Duration {
        Duration::from_secs(m * 60)
    }

    fn file(toml: &str) -> FileConfig {
        FileConfig::parse(toml).unwrap()
    }

    fn resolve(f: &FileConfig, profile: Option<&str>) -> Config {
        Config::resolve(f, profile, Settings::default()).unwrap()
    }

    #[test]
    fn empty_config_resolves_with_built_in_defaults() {
        let c = resolve(&FileConfig::default(), None);
        let d = Config::default();
        assert_eq!(c.work_duration, d.work_duration);
        assert_eq!(c.sessions_before_long_break, d.sessions_before_long_break);
    }

    #[test]
    fn profiles_inherit_unset_keys_from_the_default_profile() {
        let f = file(
            r#"
            [profiles.default]
            work = "30m"
            break = "7m"
            sessions = 3

            [profiles.deep]
            work = "50m"
            "#,
        );
        let c = resolve(&f, Some("deep"));

        assert_eq!(c.work_duration, mins(50)); // own value wins
        assert_eq!(c.break_duration, mins(7)); // inherited
        assert_eq!(c.sessions_before_long_break, 3); // inherited
        assert_eq!(c.long_break_duration, Config::default().long_break_duration); // built-in
    }

    #[test]
    fn cli_overrides_profile_which_overrides_default_profile() {
        let f = file(
            r#"
            [profiles.default]
            work = "30m"
            auto-start = true

            [profiles.deep]
            work = "50m"
            "#,
        );
        let cli = Settings {
            work: Some(mins(10)),
            auto_start: Some(false),
            ..Settings::default()
        };
        let c = Config::resolve(&f, Some("deep"), cli).unwrap();

        assert_eq!(c.work_duration, mins(10));
        assert!(!c.auto_start);
    }

    #[test]
    fn undefined_default_profile_is_allowed() {
        let f = file("[profiles.a]\nwork = \"30m\"");
        assert_eq!(
            resolve(&f, None).work_duration,
            Config::default().work_duration
        );
        assert!(Config::resolve(&f, Some("default"), Settings::default()).is_ok());
    }

    #[test]
    fn unknown_profile_is_an_error_listing_the_available_ones() {
        let f = file("[profiles.a]\n[profiles.b]");
        let err = Config::resolve(&f, Some("nope"), Settings::default()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("nope") && msg.contains("a, b, default"),
            "{msg}"
        );
    }

    #[test]
    fn dangling_default_profile_errors_unless_overridden_by_the_cli() {
        let f = file("default-profile = \"gone\"\n[profiles.a]");

        let err = Config::resolve(&f, None, Settings::default()).unwrap_err();
        assert!(err.to_string().contains("default-profile"));

        assert!(Config::resolve(&f, Some("a"), Settings::default()).is_ok());
    }

    #[test]
    fn session_durations_are_validated_like_on_the_cli() {
        assert!(FileConfig::parse("[profiles.a]\nwork = \"0s\"").is_err());
        assert!(FileConfig::parse("[profiles.a]\nwork = \"60m\"").is_err());
        assert!(FileConfig::parse("[profiles.a]\nwork = \"59m\"").is_ok());
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(FileConfig::parse("[profiles.a]\nwrk = \"5m\"").is_err());
        assert!(FileConfig::parse("work = \"5m\"").is_err());
    }

    #[test]
    fn example_config_is_valid() {
        let f = file(EXAMPLE);
        for name in f.profiles.keys() {
            resolve(&f, Some(name));
        }

        resolve(&f, None);
    }
}
