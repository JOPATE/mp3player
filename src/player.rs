use id3::{Tag, TagLike};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

enum Mode {
    LocalFiles,
    SpotifyConnect,
}

/// Holds everything needed to display the currently loaded song.
pub struct NowPlaying {
    pub artist: String,
    pub title: String,
}

/// Wraps a Sink plus the bits of state your old main() carried around manually
/// (current index, looping flag, total duration of the loaded song).
pub struct Player {
    pub sink: Arc<Mutex<Sink>>,
    _stream: OutputStream, // must stay alive for the whole program, never read directly
    pub songs: Vec<PathBuf>,
    pub current: usize,
    pub looping: bool,
    pub total_duration: Option<Duration>,
}

impl Player {
    /// Builds a new Player from a song list and immediately loads the first song.
    pub fn new(songs: Vec<PathBuf>) -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Arc::new(Mutex::new(Sink::try_new(&handle).unwrap()));

        let mut player = Player {
            sink,
            _stream: stream,
            songs,
            current: 0,
            looping: false,
            total_duration: None,
        };

        player.load_current();
        player
    }

    /// Loads whatever song `self.current` points to into the sink and reads its tags.
    /// Call this after changing `self.current` (next/prev/restart playlist).
    pub fn load_current(&mut self) {
        let path = &self.songs[self.current];

        let file = BufReader::new(File::open(path).unwrap());
        let source = Decoder::new(file).unwrap();
        self.total_duration = source.total_duration();
        self.sink.lock().unwrap().append(source);
    }

    /// Reads ID3 tags (or falls back to filename) for whatever song `self.current` points to.
    /// Kept separate from `load_current` so you can call it any time without re-decoding audio.
    pub fn now_playing(&self) -> NowPlaying {
        let path = &self.songs[self.current];
        let tag = Tag::read_from_path(path).unwrap();

        let artist_opt = tag.artist();
        let title_opt = tag.title();

        if artist_opt.is_none() && title_opt.is_none() {
            let filename = path.file_stem().unwrap().to_string_lossy().to_string();
            return NowPlaying {
                artist: String::new(),
                title: filename,
            };
        }

        let artist_raw = artist_opt.unwrap_or("Unknown artist");
        let artists: Vec<&str> = artist_raw.split('\0').map(|s| s.trim()).collect();

        NowPlaying {
            artist: artists.join(", "),
            title: title_opt.unwrap_or("Unknown title").to_string(),
        }
    }

    pub fn play_pause(&self) {
        let sink = self.sink.lock().unwrap();
        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.lock().unwrap().is_paused()
    }

    /// Skips to the next song in the queue (does not touch `self.current` -
    /// that's handled by the main loop's "sink is empty" check, same as before).
    pub fn skip(&self) {
        self.sink.lock().unwrap().skip_one();
    }

    pub fn volume(&self) -> f32 {
        self.sink.lock().unwrap().volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.lock().unwrap().set_volume(vol.clamp(0.0, 1.0));
    }

    pub fn get_pos(&self) -> Duration {
        self.sink.lock().unwrap().get_pos()
    }

    pub fn try_seek(&self, pos: Duration) {
        self.sink.lock().unwrap().try_seek(pos).unwrap();
    }

    pub fn clear(&self) {
        self.sink.lock().unwrap().clear();
    }

    pub fn is_empty(&self) -> bool {
        self.sink.lock().unwrap().empty()
    }

    pub fn speed(&self) -> f32 {
        self.sink.lock().unwrap().speed()
    }

    pub fn set_speed(&self, speed: f32) {
        self.sink.lock().unwrap().set_speed(speed);
    }

    pub fn toggle_looping(&mut self) {
        self.looping = !self.looping;
    }

    /// Goes back one song (used by your "r" key when near the start of a track).
    /// Returns false if already at the first song.
    pub fn previous(&mut self) -> bool {
        if self.current == 0 {
            return false;
        }
        self.current -= 1;
        self.clear();
        self.load_current();
        self.sink.lock().unwrap().play();
        true
    }

    /// Restarts the current song from the beginning.
    pub fn restart(&self) {
        self.try_seek(Duration::ZERO);
    }

    /// Advances to the next song index. Returns false if the playlist is finished
    /// (caller decides whether that means "stop" or "loop back to 0").
    pub fn advance(&mut self) -> bool {
        if !self.looping {
            self.current += 1;
        }
        if self.current >= self.songs.len() {
            return false;
        }
        self.load_current();
        true
    }

    /// Resets back to the first song - used when the whole playlist restarts.
    pub fn reset_to_start(&mut self) {
        self.current = 0;
        self.load_current();
    }
}
