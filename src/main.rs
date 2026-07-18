mod player;
mod playlist;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use player::Player;
use std::time::{Duration, Instant};

fn main() {
    let songs = playlist::load_songs();
    let mut player = Player::new(songs);

    println!("Controls: [space] play/pause  [n] next  [r] back  [+/-] volume [⬅︎/➡︎] rewind/advance 5s  [l] looping  [q] quit");
    let mut terminal = ratatui::init();

    let mut status = "Controls: [space] play/pause  [n] next  [r] back  [+/-] volume [⬅︎/➡︎] rewind/advance 5s  [l] looping  [q] quit [c] Nightcorenn".to_string();
    let mut status_set_at = Instant::now();
    let mut tracking_pos = 0;
    player.set_volume(0.5);
    'outer: loop {
        loop {
            if event::poll(Duration::from_millis(500)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char(' ') => {
                                player.play_pause();
                                status = if player.is_paused() {
                                    "⏸ Paused".to_string()
                                }
                                else {
                                    "▶ Resumed".to_string()
                                };
                                status_set_at = Instant::now();
                            }
                            KeyCode::Char('n') => {
                                status = "⏭ Skipped".to_string();
                                status_set_at = Instant::now();
                                player.skip();
                            }
                            KeyCode::Char('+') => {
                                let vol = (player.volume() + 0.1).min(1.0);
                                player.set_volume(vol);
                                status = format!("🔊︎ Volume: {:.0}%", vol * 100.0);
                                status_set_at = Instant::now();
                            }
                            KeyCode::Char('-') => {
                                let vol = (player.volume() - 0.1).max(0.0);
                                player.set_volume(vol);
                                status = format!("🔉︎ Volume: {:.0}%", vol * 100.0);
                                status_set_at = Instant::now();
                            }
                            KeyCode::Char('r') => {
                                let pos = player.get_pos().as_secs();
                                if pos <= 3 {
                                    if player.previous() {
                                        status = "⏮ Previous song".to_string();
                                    } else {
                                        status = "⏮ Already at first song".to_string();
                                    }
                                } else {
                                    player.restart();
                                    status = "⏮ Restarted song".to_string();
                                }
                                tracking_pos = 0;
                                status_set_at = Instant::now();
                            }
                            KeyCode::Left => {
                                let pos = player.get_pos();
                                let new_pos = pos.saturating_sub(Duration::from_secs(5));
                                player.try_seek(new_pos);
                                status = "⏪︎ Rewound 5s".to_string();
                                status_set_at = Instant::now();
                            }
                            KeyCode::Right => {
                                let pos = player.get_pos();
                                let total = player.total_duration.unwrap_or(Duration::ZERO);
                                if pos + Duration::from_secs(6) >= total {
                                    player.skip();
                                } else {
                                    let new_pos = pos + Duration::from_secs(5);
                                    player.try_seek(new_pos);
                                }
                                status = "⏩︎ Skipped 5s".to_string();
                                status_set_at = Instant::now();
                            }
                            KeyCode::Char('l') => {
                                player.toggle_looping();
                                if player.looping {
                                    status = "🔁︎ Now Looping".to_string();
                                } else {
                                    status = "🔁︎ Stopped Looping".to_string();
                                }
                                status_set_at = Instant::now();
                            }
                            KeyCode::Char('q') => {
                                ratatui::restore();
                                println!("\r👋︎ Closing!");
                                break 'outer;
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                ratatui::restore();
                                println!("\r👋︎ Closing (Ctrl+C)!");
                                break 'outer;
                            }
                            KeyCode::Char('c') => {
                                if player.speed() == 1.0 {
                                    player.set_speed(1.20);
                                    status = "🔥Nightcore ON🔥".to_string();
                                } else if player.speed() == 0.8 {
                                    player.set_speed(1.20);
                                    status = "🔥Nightcore ON🔥".to_string();
                                }
                                else {
                                    player.set_speed(1.0);
                                    status = "🥀Nightcore OFF🥀".to_string();
                                }
                                status_set_at = Instant::now();
                            }
                            _ => {}
                        }
                    }
                }
            }
            if player.speed() != 1.0 {
                tracking_pos = 1;
            }
            if status_set_at.elapsed() >= Duration::from_secs(2) {
                status.clear();
            }
            terminal.draw(|frame| {
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(3), // now playing
                        ratatui::layout::Constraint::Length(3), // status message
                        ratatui::layout::Constraint::Length(3), // progress bar
                    ])

                    .split(frame.area());

                ui::now_playing_ui(frame, &player, chunks[0]);
                frame.render_widget(ui::status_line(&status), chunks[1]);
                frame.render_widget(ui::progress_bar(&mut player, tracking_pos), chunks[2]);
            }).unwrap();
            if player.is_empty() {
                tracking_pos = 0;
                if player.advance() {} else {
                    break;
                }
            }
        }

        player.reset_to_start();
        status = "🔁 Restarting playlist...".to_string();
    }
}