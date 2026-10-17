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
| `--face <FACE>` | Watch face style: `large`, `medium`, or `small` | `large` |
| `--auto-start` / `--no-auto-start` | Start immediately and keep running between sessions | `--no-auto-start` |
| `--notify` / `--no-notify` | Desktop notifications | `--notify` |

Controls: `[space]` start/stop, `[r]` reset, `[s]` skip, `[a]` toggle auto-start, `[q]` quit.

### Watch faces

The timer can be drawn as a large, medium, or small block-character watch face:

| Face | Size | Pixel style |
| --- | --- | --- |
| `large` | 40 × 8 | Full |
| `medium` | 20 × 4 | Quadrant |
| `small` | 20 × 3 | Sextant |

If the selected face does not fit the terminal, the timer falls back to plain `MM:SS` text. The face is selected at startup with `--face` and cannot be changed while the app is running.

### Examples

```sh
# 50/10 with a long break every 3 sessions
concentrated --work 50m --break 10m --long-break 30m --sessions 3

# Never take a long break
concentrated --sessions 0
```
