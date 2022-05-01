use crate::config::{ Config };
use crate::monitor::{ Monitor, create as create_monitor };

use std::net::{ TcpListener, TcpStream };
use std::collections::{ HashMap };

use std::io::{ BufReader, BufWriter, BufRead, Write };
use either::{ Either };

use thread_control::{ make_pair };

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
            let monitor_name = command.strip_suffix("?").unwrap();
            match self.monitors.get(monitor_name) {
                Some(monitor) => {
                    // FIXME write monitor info as result
                    //  Single value sensors return [ description, min, max, unit ].join("\t")
                    //   if min & max are 0, the UI is auto-ranged, the unit is optional (e.g. pscount)
                    //  Multivalue sensors return a table spec in two lines
                    //      -> tab-delimited headers and associated unit spec
                    //       d: integer value
                    //       D: integer value that should be localized in the frontend
                    //       f: floating point value
                    //       s: string value
                    //       S: string value that needs to be translated
                    //          Strings must be added to the ProcessList::columnDict dictionary.
                },
                None => {}
            }
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
                    protocol.handle_client(&socket);
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
