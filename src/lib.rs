#[allow(clippy::all)]
pub mod generated_code {
    slint::include_modules!();
}
use std::path::PathBuf;

pub use generated_code::*;
use serde::{Serialize, Deserialize};

pub mod player_work;
pub mod loadfile;

#[derive(Debug, Clone,  PartialEq, Serialize, Deserialize,Default)]
pub struct Song {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: Option<f64>,
    date: Option<String>,
    playing: bool,
    path: PathBuf,
}
impl Song {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            title: None,
            artist: None,
            album: None,
            date: None,
            duration: None,
            playing: false,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn set_title(&mut self, title: Option<&str>) -> Self {
        if let Some(title) = title {
            self.title = Some(title.to_string());
        }

        self.to_owned()
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn set_artist(&mut self, artist: Option<&str>) -> Self {
        if let Some(artist) = artist {
            self.artist = Some(artist.to_string());
        }
        self.to_owned()
    }

    pub fn artist(&self) -> Option<String> {
        self.artist.clone()
    }

    pub fn set_album(&mut self, album: Option<&str>) -> Self {
        if let Some(album) = album {
            self.album = Some(album.to_string());
        }
        self.to_owned()
    }

    pub fn album(&self) -> Option<String> {
        self.album.clone()
    }

    pub fn set_date(&mut self, date: Option<String>) -> Self {
        self.date = date;
        self.to_owned()
    }

    pub fn date(&self) -> Option<String> {
        self.date.clone()
    }

   
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayList {
    songs: Vec<Song>,
}

impl PlayList {
    pub fn new() -> Self {
        Self {
            songs:vec![]
        }
    }
    pub fn songs(&self) -> Vec<Song> {
        self.songs.clone()
    }
}

