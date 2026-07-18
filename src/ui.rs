use ratatui::{
    widgets::{Paragraph, Gauge},
    Frame,
};
use crate::player::Player;
use std::time::Duration;

pub fn now_playing_ui(frame: &mut Frame, player: &Player, area: ratatui::layout::Rect) {
    let now_playing = player.now_playing();

    let text = if now_playing.artist.is_empty() {
        now_playing.title
    } else {
        format!("{} - {}", now_playing.artist, now_playing.title)
    };

    let paragraph = Paragraph::new(text).centered();

    frame.render_widget(paragraph, area);
}

pub fn progress_bar(player: &mut Player, tracking_pos: i32) -> Gauge<'_> {
    let pos = player.get_pos();
    let total = player.total_duration.unwrap_or(Duration::ZERO);

    let ratio = if tracking_pos == 1 {
        1.0
    } else if total.as_secs_f64() > 0.0 {
        (pos.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut label = format!(
        "{:02}:{:02} / {:02}:{:02}",
        pos.as_secs() / 60, pos.as_secs() % 60,
        total.as_secs() / 60, total.as_secs() % 60
    );
    if tracking_pos == 1 {
        label = "Position is cooked".to_string();
    }

    Gauge::default()
        .ratio(ratio)
        .label(label)
}

pub fn status_line(message: &str) -> Paragraph<'_> {
    Paragraph::new(message.to_string()).centered()
}

