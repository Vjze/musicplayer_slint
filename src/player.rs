use std::io::BufReader;

use rodio::Source;

use crate::{Song, PlayList};


pub async fn paly_song(v:Vec<Song>) -> tokio::io::Result<()> {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let play_sink = rodio::Sink::try_new(&handle).unwrap();
    for i in v{
        println!("ac");
    
        let file = std::fs::File::open(i.path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        play_sink.append(source);
        // play_sink.sleep_until_end()
    }
    Ok(())
}