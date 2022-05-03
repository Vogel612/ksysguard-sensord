use crate::config::{ Config };
use crate::protocol::{ Commands };
use crate::monitor::{ create as create_monitor };

use std::net::{ TcpListener };
use std::collections::{ HashMap };

use std::io::{ BufReader, BufWriter };

use thread_control::{ make_pair };


const DEFAULT_BIND_SPEC: &str = "0.0.0.0";

pub struct Sensord {}

impl Sensord {
    pub fn start(config: &Config) {
        let mut known_monitors = HashMap::new();
        for monitor_definition in config.monitors.iter() {
            known_monitors.insert(monitor_definition.name.to_owned(), create_monitor(&monitor_definition));
        }

        // establish listening socket
        let listener = TcpListener::bind((DEFAULT_BIND_SPEC, config.port))
            .expect("Failed to bind listening port to accept connections.");
        let protocol = Commands {
            monitors: &known_monitors,
        };
        let mut join_handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
        let (flag, control) = make_pair();
        let flag_ptr = std::sync::Arc::new(Option::Some(flag));
        for mon in known_monitors.values() {
            match mon.start(flag_ptr.clone()) {
                Some(handle) => { join_handles.push(handle) }
                None => {}
            }
        }
        loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    let mut reader = BufReader::new(&socket);
                    let mut writer = BufWriter::new(&socket);
                    protocol.handle_client(&mut reader, &mut writer);
                    break;
                },
                Err(_e) => {
                    println!("Failed to accept a connection to the TCP listen port!");
                }
            }
        }

        control.stop();
        for h in join_handles {
            h.join();
        }
    }
}
