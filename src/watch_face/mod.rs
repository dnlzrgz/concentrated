mod digital;

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
};

use crate::app::Model;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)]
pub enum WatchFace {
    #[default]
    #[value(name = "digital-large")]
    DigitalLarge,
    #[value(name = "digital-medium")]
    DigitalMedium,
    #[value(name = "digital-small")]
    DigitalSmall,
}

impl WatchFace {
    pub fn render(self, frame: &mut Frame, area: Rect, model: &Model) {
        digital::render(frame, area, model, self);
    }
}

fn timer_text(model: &Model) -> String {
    let secs_left = model.remaining().as_secs();
    format!("{:02}:{:02}", secs_left / 60, secs_left % 60)
}

fn centered_vertically(area: Rect, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    row
}
