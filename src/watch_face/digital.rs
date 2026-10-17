use ratatui::{Frame, layout::Rect};
use tui_big_text::{BigText, PixelSize};

use super::WatchFace;
use crate::{
    app::Model,
    watch_face::{centered_vertically, timer_text},
};

fn pixel_size(face: WatchFace) -> PixelSize {
    match face {
        WatchFace::Large => PixelSize::Full,
        WatchFace::Medium => PixelSize::Quadrant,
        WatchFace::Small => PixelSize::Sextant,
    }
}

fn glyph_height(face: WatchFace) -> u16 {
    match face {
        WatchFace::Large => 8,
        WatchFace::Medium => 4,
        WatchFace::Small => 3,
    }
}

pub(super) fn render(frame: &mut Frame, area: Rect, model: &Model, face: WatchFace) {
    let timer = BigText::builder()
        .pixel_size(pixel_size(face))
        .centered()
        .lines(vec![timer_text(model).into()])
        .build();

    frame.render_widget(timer, centered_vertically(area, glyph_height(face)));
}
