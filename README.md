# concentrated

`concentrated` is another pomodoro application built with [ratatui](https://ratatui.rs) and following [the Elm architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/). Right now there is nothing special about it and you'll probably be better off using any other alternative out there or just building your own.

## Usage

```sh
concentrated [OPTIONS]
```

By default a "cycle" is four 25-minute work sessions separated by 5-minute breaks, followed by a 15-minute long break (the classical Pomodoro).

| Option | Description | Default |
| --- | --- | --- |
| `--work <DURATION>` | Work session duration, e.g. `25m` | 25m |
| `--break <DURATION>` | Short break duration, e.g. `5m` | 5m |
| `--long-break <DURATION>` | Long break duration, e.g. `15m` | 15m |
| `--sessions <N>` | Work sessions before a long break (`0` disables long breaks) | 4 |
| `--face <FACE>` | Watch face style: `digital-large`, `digital-medium`, or `digital-small` | `digital-large` |
| `--auto-start` / `--no-auto-start` | Start immediately and keep running between sessions | `--no-auto-start` |
| `--notify` / `--no-notify` | Desktop notifications | `--notify` |
| `-p, --profile <NAME>` | Config [profile](#configuration) to use | `default-profile` from the config, else `default` |
| `--list-profiles` | List the available profiles and exit | |
| `-c, --config <PATH>` | Config file to use | see [Configuration](#configuration) |

Controls: `[space]` start/stop, `[r]` reset, `[s]` skip, `[a]` toggle auto-start, `[q]` quit.

## Configuration

Settings can be stored in a TOML file and grouped into named **profiles**, e.g. `work` or `deep-work`. The file is optional and is looked up at:

| Platform | Path |
| --- | --- |
| Linux, macOS | `$XDG_CONFIG_HOME/concentrated/config.toml`, or `~/.config/concentrated/config.toml` |
| Windows | `%APPDATA%\concentrated\config.toml` |

Use `--config <PATH>` to read a different file.

```toml
# Profile used when --profile isn't used (default: "default").
default-profile = "work"

# Shared
[profiles.default]
work = "25m"
break = "5m"
long-break = "15m"
sessions = 4
face = "digital-large"
auto-start = false
notify = true

[profiles.work]
work = "45m"
break = "10m"

[profiles.deep-work]
work = "55m"
break = "15m"
sessions = 2
```

See [`config.example.toml`](config.example.toml).

- Profile keys have the same names and values as the command-line options (`--long-break` is `long-break`, `--notify`/`--no-notify` is `notify = true`/`false`). All of them are optional, and durations are limited to 1s to 59m.
- `[profiles.default]` is optional. If it's missing, the built-in defaults are the base.
- Unknown keys or types are treated as errors (e.g. `wrk` instead of `work`).
- `on-session-end` has no command-line option, see [Hooks](#hooks).

### Precedence

From lowest to highest:

1. Built-in defaults
2. `[profiles.default]`
3. The selected profile: `--profile`, else `default-profile`, else `default`
4. Command-line options, e.g. `concentrated -p work --break 3m` uses the `work` profile with a 3-minute break

```sh
concentrated -p deep-work
concentrated --list-profiles
```

### Hooks

`on-session-end` is a shell command that runs when a session's timer runs out (e.g. to play a sound). Like any other key it can be set per profile and is inherited from `[profiles.default]`; an empty string (`""`) disables an inherited hook. Failures are silent.

### Watch faces

The timer can be drawn as a large, medium, or small block-character watch face:

| Face | Size | Pixel style |
| --- | --- | --- |
| `digital-large` | 40 × 8 | Full |
| `digital-medium` | 20 × 4 | Quadrant |
| `digital-small` | 20 × 3 | Sextant |

If the selected face does not fit the terminal, the timer falls back to plain `MM:SS` text. The face is selected at startup with `--face` (or `face` in a profile).

### Examples

```sh
# 50/10 with a long break every 3 sessions
concentrated --work 50m --break 10m --long-break 30m --sessions 3

# Never take a long break
concentrated --sessions 0
```
