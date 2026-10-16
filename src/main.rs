use std::time::{Duration, Instant};

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use notify_rust::Notification;
use ratatui::DefaultTerminal;

use concentrated::{app::Model, cli::Cli, config::Config, message::Message, ui};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let config = Config::resolve(&cli);

    let terminal = ratatui::init();
    let result = run(terminal, config);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, config: Config) -> Result<()> {
    let mut model = Model::new(config);
    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    while !model.should_quit() {
        terminal.draw(|frame| ui::view(&model, frame))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && let Some(msg) = handle_key(key.code)
        {
            model.update(msg)?;
        }

        if last_tick.elapsed() >= tick_rate {
            let prev_session = model.session_label();
            model.update(Message::Tick)?;
            if model.session_label() != prev_session && config.notifications {
                notify_session_end(prev_session)?;
            }

            last_tick += tick_rate;
        }
    }

    Ok(())
}

fn handle_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
        KeyCode::Char(' ') | KeyCode::Enter => Some(Message::ToggleStartStop),
        KeyCode::Char('s') => Some(Message::Skip),
        KeyCode::Char('r') => Some(Message::Reset),
        KeyCode::Char('a') => Some(Message::ToggleAutoStart),
        _ => None,
    }
}

fn notify_session_end(finished_session: &str) -> Result<()> {
    Notification::new()
        .summary("Pomodoro")
        .body(&format!("{finished_session} session finished!"))
        .show()?;

    Ok(())
}
