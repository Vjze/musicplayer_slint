#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use musicplayer_slint::{App, player_work};
use slint::ComponentHandle;

fn main() {
    let app = App::new().unwrap();
    let cargo_worker = player_work::PlayerWorker::new(&app);

    app.on_loadfile({
        let cargo_channel = cargo_worker.channel.clone();
        move || {
                cargo_channel
                    .send(player_work::CtrlMessage::LoadFile)
                    .unwrap()
        
        }
    });
    app.on_paly({
        let cargo_channel = cargo_worker.channel.clone();
        move || {
                cargo_channel
                    .send(player_work::CtrlMessage::Play)
                    .unwrap()
        
        }
    });
    app.run().unwrap();
    cargo_worker.join().unwrap();
}