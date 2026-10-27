use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let area = frame.area();
    let [timer_area, controls_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

    let mut parts = vec![
        model.session_label(),
        if model.is_running() {
            "running"
        } else {
            "paused"
        },
    ];

    if model.auto_start() {
        parts.push("auto-start");
    }

    let session_title = format!(" {} ", parts.join(" · "));
    let mut timer_block = Block::default()
        .title(session_title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if let Some((current, total)) = model.session_progress() {
        timer_block =
            timer_block.title_bottom(Line::from(format!(" {current}/{total} ")).centered());
    }

    let inner_area = timer_block.inner(timer_area);
    frame.render_widget(timer_block, timer_area);

    model.watch_face().render(frame, inner_area, model);

    let controls =
        Paragraph::new("[space] start/stop · [r] reset · [s] skip · [a] auto-start · [q] quit")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

    frame.render_widget(controls, controls_area);
}
