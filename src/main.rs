use std::{
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use notify_rust::Notification;
use ratatui::DefaultTerminal;

use concentrated::{
    app::Model,
    cli::Cli,
    config::{Config, FileConfig},
    hooks,
    message::Message,
    ui,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let file = FileConfig::load(cli.config.as_deref())?;

    if cli.list_profiles {
        print!("{}", file.list_profiles(cli.profile.as_deref()));
        return Ok(());
    }

    let config = Config::resolve(&file, cli.profile.as_deref(), cli.settings())?;

    let terminal = ratatui::init();
    let result = run(terminal, config);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, config: Config) -> Result<()> {
    let mut model = Model::new(config.clone());
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
            let was_running = model.is_running();
            model.update(msg)?;
            if !was_running && model.is_running() {
                last_tick = Instant::now();
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Some(end) = model.update(Message::Tick)? {
                if config.notifications {
                    notify_session_end(&end.finished.replace('-', " "));
                }
                if let Some(command) = &config.on_session_end {
                    hooks::run(command, end);
                }
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

/// Errors are ignored if no daemon is running (SSH, container, etc). Uses a thread
/// in case the daemon is slow.
fn notify_session_end(finished_session: &str) {
    let body = format!("{finished_session} session finished!");
    thread::spawn(move || {
        let _ = Notification::new().summary("Pomodoro").body(&body).show();
    });
}
