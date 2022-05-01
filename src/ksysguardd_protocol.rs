use crate::config::{ Config };
use crate::monitor::{ Monitor, create as create_monitor };

// use tokio::net::{ TcpListener, TcpStream };
use std::net::{ TcpListener, TcpStream };
use std::collections::{ HashMap };

use std::io::{ BufReader, BufWriter, BufRead, Write };
use either::{ Either };

const UNKNOWN_COMMAND: &str = "UNKNOWN COMMAND";
const MESSAGE_END: &str = "\nksysguardd> ";
const DEFAULT_BIND_SPEC: &str = "0.0.0.0";

enum Exit {
    Exit,
}

struct Commands<'a> {
    // a monitor has multiple sensors?!
    //   (like "nvidia-smi dmon" contains multiple information points)
    // it maintains their metainformation (datatype, value range, ...)
    // for monitoring sensors it retains their latest value
    // for polling sensors it is responsible for getting their value
    pub monitors: &'a HashMap<String, Box<dyn Monitor>>,
}

// required commands:
// monitors, quit
// a description for a sensor
// recommended "commands":
// test, specific sensors involve sensors
static PROTOCOL_COMMANDS: &[&str] = &[
    "test", "quit", "monitors"
];

impl Commands<'_> {
    fn on_command(&self, input: &str) -> Either<String, Exit> {
        let command = input.trim();
        if command.eq_ignore_ascii_case("quit") {
            // terminate!
            return either::Right(Exit::Exit);
        }
        if command.starts_with("test") {
            match command.split_once(" ") {
                Some((_ ,checked)) => {
                    if PROTOCOL_COMMANDS.contains(&checked) ||
                        self.monitors.contains_key(checked) {
                        return either::Left("1".into());
                    }
                },
                None => {}
            }
            return either::Left("0".into());
        }
        if command.starts_with("monitors") {
            return either::Left(itertools::join(self.monitors.keys(), "\n"));
        }
        let is_info_query = command.ends_with("?");
        if is_info_query {
            // currently unsupported
        } else {
            if self.monitors.contains_key(command) {
                let mon = self.monitors.get(command).unwrap();
                return either::Left(mon.get_value());
            }
        }
        // attempt to find the monitor specified, give information if terminated by ?
        // return UNKNOWN COMMAND if no matching monitor is found
        return either::Left(UNKNOWN_COMMAND.into());
    }

    // TODO "RECONFIGURE" over stderr??
    fn handle_client(&self, socket: & TcpStream) {

        let mut reader = BufReader::new(socket);
        let mut writer = BufWriter::new(socket);
        writer.write(MESSAGE_END.as_bytes())
            .and(writer.flush());
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            let bytes = reader.read_line(&mut line_buf);
            match bytes {
                Ok(0) => {
                    // EOF 
                    break;
                },
                Ok(_) => {
                    let result = self.on_command(&* line_buf);
                    match result {
                        either::Left(response) => {
                            // compiler wants us to unwrap this and handle write errors
                            writer.write(response.as_bytes())
                                .and(writer.write(MESSAGE_END.as_bytes()))
                                .and(writer.flush());
                        },
                        either::Right(_) => {
                            // or alternatively exit?
                            break;
                        }
                    }
                },
                Err(_e) => {
                    // TODO handle error when reading from socket
                }
            }
        }
    }
}

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
        for mon in known_monitors.values() {
            match mon.start() {
                Some(handle) => { join_handles.push(handle) }
                None => {}
            }
        }
        loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    protocol.handle_client(&socket);
                    break;
                },
                Err(_e) => {
                    println!("Failed to accept a connection to the TCP listen port!");
                }
            }
        }

        for h in join_handles {
            // FIXME interrupt / gracefully shut down monitor threads
            h.join();
        }
    }
}
