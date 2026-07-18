use std::fs;
use std::path::PathBuf;

/// Loads all .mp3 files from the user's `~/Music/mp3test` folder, sorted alphabetically.
pub fn load_songs() -> Vec<PathBuf> {
    let music_dir = dirs::home_dir()
        .expect("could not find home dir")
        .join("Music")
        .join("mp3test");

    let mut songs: Vec<_> = fs::read_dir(&music_dir)
        .expect("could not read music dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()?.to_str()? == "mp3" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    songs.sort();
    songs
}