use std::{
    io::BufReader,
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
};

use crate::{
    generated_code::App, loadfile::run_load,  Song, PlayList, 
};
use futures::future::{Fuse, FutureExt};
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub enum CtrlMessage {
    LoadFile,
    Play,
    Stop,
    Pause,
    Quit,
}

pub struct PlayerWorker {
    pub channel: UnboundedSender<CtrlMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl PlayerWorker {
    pub fn new(query: &App) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = query.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(player_worker_loop(r, handle_weak))
                    .unwrap()
            }
        });
        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(CtrlMessage::Quit);
        self.worker_thread.join()
    }
}

async fn player_worker_loop(
    mut r: UnboundedReceiver<CtrlMessage>,
    handle: slint::Weak<App>,
) -> tokio::io::Result<()> {
    let (tx, rx) = channel();
    let mut init_state = AppState {
        playlists: Vec::new(),
    };
    let mut playlist = PlayList::new();
    let run_load_future = Fuse::terminated();
    let run_play_future = Fuse::terminated();
    futures::pin_mut!(run_load_future, run_play_future,);
    loop {
        let m = futures::select! {
                res = run_load_future => {
                    playlist.songs = res?;
                    continue;
                }
                res = run_play_future => {
                    res?;
                    continue;
                }


            m = r.recv().fuse() => {
                match m {
                    None => return Ok(()),
                    Some(m) => m,
                }
            }
        };
        match m {
            CtrlMessage::LoadFile => run_load_future.set(run_load(handle.clone()).fuse()),
            CtrlMessage::Quit => return Ok(()),
            CtrlMessage::Play => {
                // let list = init_state.playlists.clone();
                let list = playlist.songs();
                let sta = MusicCommand::Play;
                let _ = tx.send(sta);
                run_play_future.set(play(list, &rx).fuse())
                // run_play_future.set(add_song(init_state).fuse())
            }
            CtrlMessage::Stop => {
                let sta = MusicCommand::Stop;
                let _ = tx.send(sta);
            },
            CtrlMessage::Pause => {
                let sta = MusicCommand::Pause;
                let _ = tx.send(sta);
            },
        }
        
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    #[serde(skip_serializing, skip_deserializing)]
    pub playlists: Vec<Song>,
}




async fn play(v: Vec<Song>, rx: &Receiver<MusicCommand>) -> tokio::io::Result<()> {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    let init = AppState {
        playlists: v,
    };
    for i in init.playlists {
        let mut state = MusicState::Playing;
        loop{
            if let Ok(cmd) = rx.try_recv() {
                match cmd {
                    MusicCommand::Pause => {
                        state = MusicState::Paused;
                        sink.pause()
                    }
                    MusicCommand::Play => {
                        state = MusicState::Playing;
                        sink.play()
                    }
                    MusicCommand::Stop => {
                        state = MusicState::Stoped;
                        sink.stop()
                    },
                }
            }

            if state == MusicState::Playing {
                let file = std::fs::File::open(i.path()).unwrap();
                let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                sink.append(source);
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum MusicCommand {
    Pause,
    Play,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicState {
    Playing,
    Paused,
    Stoped,
}
