use std::{borrow::BorrowMut, sync::{Arc, Mutex}};

use crate::{generated_code::App, loadfile::run_load, Song, PlayList, player::paly_song, Status};
use futures::future::{Fuse, FutureExt};
use rodio::OutputStreamHandle;
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub enum CtrlMessage {
    LoadFile,
    Play,
    // SortDescending { index: i32 },
    // SortAscending { index: i32 },
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
    let (_stream, handle_o) = rodio::OutputStream::try_default().unwrap();
    let play_sink = Arc::new(Mutex::new(rodio::Sink::try_new(&handle_o).unwrap()));
    let mut init_state = AppState {
        app_status: Arc::new(Mutex::new(Status::Stop)),
        play_lists: Vec::new(),
        current_song: Arc::new(Mutex::new(Song::default())),
        volume: 0.3,
        progress_rate: 0.5,
        current_play_list: vec![],
        sink: play_sink,
        stream: Arc::new(handle_o),
    };
    let run_load_future = Fuse::terminated();
    let run_play_future = Fuse::terminated();
    futures::pin_mut!(
        run_load_future,
        run_play_future,
    );
    loop {
        let m = futures::select! {
                res = run_load_future => {
                    init_state.current_play_list = res?;
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
            CtrlMessage::LoadFile  => {
                run_load_future.set(run_load(handle.clone()).fuse())
            }
            CtrlMessage::Quit => return Ok(()),
            CtrlMessage::Play => {
                let list = init_state.current_play_list.clone();
                run_play_future.set(paly_song(list).fuse())
                
            },
            
        }
    }
}

#[derive(Clone)]
struct AppState {
    app_status: Arc<Mutex<Status>>,
    play_lists: Vec<PlayList>,
    current_song: Arc<Mutex<Song>>,
    sink: Arc<Mutex<rodio::Sink>>,
    progress_rate: f64,
    current_play_list: Vec<Song>,
    volume: f64,
    stream: Arc<OutputStreamHandle>,
}


