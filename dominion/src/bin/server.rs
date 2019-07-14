// server.rs
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc;

use dominion::protocol;

enum SimulationEvent {
    Join(TcpStream)

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    let (sim_tx, sim_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut players: HashMap<SocketAddr, TcpStream> = HashMap::new();

        for event in sim_rx {
            match event {
                SimulationEvent::Join(stream) => {
                    for (_addr, stream) in players.iter_mut() {
                        stream.write_all(&protocol::Tag::Join.into_bytes()).unwrap();
                    }
                    players.insert(stream.peer_addr().unwrap(), stream);
                },
            }
        }
    });

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                sim_tx.send(SimulationEvent::Join(stream)).unwrap();
            }
            Err(e) => {
                eprintln!("connection failed {}", e);
            }
        }
    }

    Ok(())
}
