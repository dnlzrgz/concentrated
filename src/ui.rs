use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_big_text::BigText;

use crate::app::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let area = frame.area();
    let [timer_area, controls_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

    let mut timer_block = Block::default()
        .title(format!(" {} ", model.session_label()))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if let Some((current, total)) = model.session_progress() {
        timer_block =
            timer_block.title_bottom(Line::from(format!(" {current}/{total} ")).centered());
    }

    let inner_area = timer_block.inner(timer_area);
    frame.render_widget(timer_block, timer_area);

    let [_, big_text_row, _, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(8),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner_area);

    frame.render_widget(timer_big_text(model), big_text_row);

    let auto_start = if model.auto_start() { "on" } else { "off" };
    let controls =
        Paragraph::new("[space] start/stop · [r] reset · [s] skip · [a] auto-start · [q] quit")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title_top(Line::from(format!(" {auto_start} ")).left_aligned()),
            );

    frame.render_widget(controls, controls_area);
}

fn timer_big_text(model: &Model) -> BigText<'static> {
    let secs_left = model.remaining().as_secs();
    let timer_text = format!("{:02}:{:02}", secs_left / 60, secs_left % 60);
    let mut style = Style::new();
    if !model.is_running() {
        style = style.add_modifier(Modifier::SLOW_BLINK);
    }

    BigText::builder()
        .centered()
        .style(style)
        .lines(vec![timer_text.into()])
        .build()
}
