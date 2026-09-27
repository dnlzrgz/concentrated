use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use notify_rust::Notification;
use ratatui::DefaultTerminal;

use concentrated::{app::Model, message::Message, ui};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut model = Model::new();
    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    while !model.should_quit() {
        terminal.draw(|frame| ui::view(&model, frame))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(msg) = handle_key(key.code) {
                        model.update(msg)?;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let prev_session = model.session_label();
            model.update(Message::Tick)?;
            if model.session_label() != prev_session {
                notify_session_end(prev_session)?;
            }

            last_tick = Instant::now();
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
