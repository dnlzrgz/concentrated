# concentrated

`concentrated` is just another pomodoro application built with [ratatui](https://ratatui.rs) and following [the Elm architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/). Right now there is nothing special about it and you'll probably be better off using any other alternative out there.

## Usage

```sh
concentrated [OPTIONS]
```

By default a "cycle" is four 25-minute work sessions separated by 5-minute breaks, followed by a 15-minute long break.

| Option | Description | Default |
| --- | --- | --- |
| `--work <DURATION>` | Work session duration, e.g. `25m` | 25m |
| `--break <DURATION>` | Short break duration, e.g. `5m` | 5m |
| `--long-break <DURATION>` | Long break duration, e.g. `15m` | 15m |
| `--sessions <N>` | Work sessions before a long break (`0` disables long breaks) | 4 |
| `--auto-start` / `--no-auto-start` | Start immediately and keep running between sessions | `--no-auto-start` |
| `--notify` / `--no-notify` | Desktop notifications | `--notify` |

Controls: `[space]` start/stop, `[r]` reset, `[s]` skip, `[a]` toggle auto-start, `[q]` quit.

### Examples

```sh
# 50/10 with a long break every 3 sessions
concentrated --work 50m --break 10m --long-break 30m --sessions 3

# Never take a long break
concentrated --sessions 0
```
