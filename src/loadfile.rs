use std::{path::Path, fs, rc::Rc};
use ffmpeg_next as ffmpeg;
use slint::{StandardListViewItem, VecModel};
use crate::{generated_code::App, Song, PlayList};



pub async fn run_load(handle: slint::Weak<App>) ->tokio::io::Result<Vec<Song>> {
    // if let Some(lib_path) = rfd::FileDialog::new().pick_folder(){
    //     let mut new_library = PlayList::new(lib_path.clone());
    //     std::thread::spawn(move || {
        let lib_path = rfd::FileDialog::new().pick_folder().unwrap();
            let songs = load_files(lib_path.display().to_string().as_str());
            let song = songs.clone();
            handle
                .upgrade_in_event_loop(move |ui| {
                    
                    let mut row_data: Vec<slint::ModelRc<StandardListViewItem>> = vec![];
                    for s in song{
                        let items = Rc::new(VecModel::default());
                        let title =  StandardListViewItem::from(slint::format!(
                            "{}",s.title.unwrap_or("?".to_string())));
                        let artist =  StandardListViewItem::from(slint::format!(
                            "{}",s.artist.unwrap_or("?".to_string())));
                        let album =  StandardListViewItem::from(slint::format!(
                            "{}",s.album.unwrap_or("?".to_string())));
                        let date =  StandardListViewItem::from(slint::format!(
                            "{}",s.date.unwrap_or("?".to_string())));
                        items.push(title);
                        items.push(artist);
                        items.push(album);
                        items.push(date);
                        row_data.push(items.into())
                    }
                    let data = Rc::new(VecModel::from(row_data));
                    ui.set_list(data.into())
                    
                }).unwrap();
            // for item in &songs {
            //     new_library.add_song(item.clone());
            // }
        // });
    // };
    Ok(songs)
}
fn load_files(dir: &str) -> Vec<Song> {
    let mut songs = vec![];
    let dir = Path::new(dir);
    // 读取当前目录下的音乐文件。
    let mut files: Vec<String> = fs::read_dir(dir)
        .ok()
        .unwrap()
        .map(|res| res.ok().map(|e| e.path().display().to_string()))
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| is_music_file(x))
        .collect();

    // 读取目录下的子目录的音乐文件
    if let Ok(other_dirs) = fs::read_dir(dir) {
        for other in other_dirs {
            if let Ok(d) = other {
                if d.path().is_dir() {
                    fs::read_dir(d.path())
                        .ok()
                        .unwrap()
                        .map(|res| res.ok().map(|e| e.path().display().to_string()))
                        .into_iter()
                        .map(|x| x.unwrap())
                        .filter(|x| is_music_file(x))
                        .for_each(|f| songs.push(get_song_meta(&f)))
                }
            }
        }
    }

    files.sort();
    for i in &files {
        let s = get_song_meta(i);
        songs.push(s);
    }
    songs
}

fn get_song_meta(f: &str) -> Song {
    let mut song = Song::default();
    ffmpeg::init().unwrap();

    match ffmpeg::format::input(&Path::new(f)) {
        Ok(context) => {
            let mut is_has_title = false;
            for (k, v) in context.metadata().iter() {
                let k_lower = k.to_lowercase();
                // 跳过???乱码的值
                if v.starts_with("?") {
                    continue;
                }
                match k_lower.as_str() {
                    "title" => {
                        song.title = Some(v.to_string());
                        is_has_title = true
                    }
                    "album" => song.album = Some(v.to_string()),
                    "artist" => song.artist = Some(v.to_string()),
                    "date" => song.date = Some(v.to_string()),
                    _ => {}
                }
            }
            if !is_has_title {
                song.title = {
                    let split_strs: Vec<&str> = f.split("/").collect();
                    let mut name: String = split_strs.last().unwrap().to_string();
                    let music_exts: Vec<&str> = vec![".flac", ".mp3", ".wav", ".m4a", ".ogg"];
                    for ext in music_exts {
                        name = name.trim_end_matches(ext).to_owned()
                    }
                    Some(name)
                }
            }
            song.duration =
                Some((context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)).round());
        }
        Err(error) => println!("error:{}", error),
    }

    song.path = f.to_string().into();
    song
}

fn is_music_file(f: &str) -> bool {
    let music_exts: Vec<&str> = vec![".flac", ".mp3", ".wav", ".m4a", ".ogg"];
    for x in &music_exts {
        if f.ends_with(x) {
            return true;
        }
    }
    return false;
}