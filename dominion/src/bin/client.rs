use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use dominion::protocol;

#[derive(Debug)]
enum State {
    Disconnected,
    Connected(thread::JoinHandle<()>),
}

struct Game {}

static SHOULD_DISCONNECT: AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn should_disconnect() -> bool {
    SHOULD_DISCONNECT.load(atomic::Ordering::SeqCst)
}

fn reset_disconnect() {
    SHOULD_DISCONNECT.store(false, atomic::Ordering::SeqCst);
}

fn signal_disconnect() {
    SHOULD_DISCONNECT.store(true, atomic::Ordering::SeqCst);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::Disconnected;

    let mut shared = Arc::new(Mutex::new(Game {}));

    let stdin = io::stdin();
    'main: for line in stdin.lock().lines() {
        let line = line.unwrap();
        state = match state {
            State::Disconnected => match line.as_str() {
                "connect" => match TcpStream::connect("127.0.0.1:8000") {
                    Ok(mut stream) => {
                        reset_disconnect();

                        State::Connected(thread::spawn({
                            let mut shared = shared.clone();
                            move || {
                                stream.set_nonblocking(true).unwrap();

                                'thread: while should_disconnect() == false {
                                    let mut tag = [0u8; 4];
                                    match stream.read(&mut tag) {
                                        Ok(0) => {
                                            eprintln!("Disconnecting from server.");
                                            break 'thread;
                                        }
                                        Ok(4) => {
                                            match protocol::Tag::try_from_bytes(tag).unwrap() {
                                                protocol::Tag::Join => {
                                                    println!("A new player has joined!");
                                                }
                                            }
                                        }
                                        Ok(_) => {
                                            eprintln!("Server sent incomplete message.");
                                            break 'thread;
                                        }
                                        Err(err) => match err.kind() {
                                            std::io::ErrorKind::WouldBlock => {
                                                std::thread::yield_now();
                                                continue 'thread;
                                            }
                                            _ => {
                                                eprintln!("Failed to read: {:?}", err);
                                                break 'thread;
                                            }
                                        },
                                    }
                                }
                            }
                        }))
                    }
                    Err(error) => {
                        eprintln!("Failed to connect: {:?}", error);
                        state
                    }
                },
                "exit" => {
                    break 'main;
                }
                _ => {
                    eprintln!("Invalid command.");
                    state
                }
            },
            State::Connected(ref connection) => match line.as_str() {
                "disconnect" => {
                    signal_disconnect();
                    State::Disconnected
                }
                _ => {
                    eprintln!("Invalid command.");
                    state
                }
            },
        };

        println!("state: {:?}", state);
    }

    Ok(())
}
