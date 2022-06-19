use crate::monitor::SensorMeta::{ Single, Multi };
use crate::monitor::{ Monitor, AsOutput };

use either::{ Either };
use std::io::{ BufRead, Write };
use std::collections::{ HashMap };


const UNKNOWN_COMMAND: &str = "UNKNOWN COMMAND";
const GREETING: &str = "";
const MESSAGE_END: &str = "\nksysguardd> ";

#[derive(Debug, PartialEq)]
enum Exit {
    Exit,
}

pub struct Commands<'a> {
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
                    match monitor.meta() {
                        Single(sv_meta) => {
                            return Either::Left(sv_meta.as_output().to_string())
                        }
                        Multi(mv_meta) => {
                            return Either::Left(mv_meta.as_output().to_string())
                        }
                    }
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

    pub fn handle_client<I, O>(&self, input: &mut I, output: &mut O) -> ()
        where I: BufRead,
        O: Write
    {
        output.write(GREETING.as_bytes())
            .and(output.write(MESSAGE_END.as_bytes()))
            .and(output.flush());
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            let bytes = input.read_line(&mut line_buf);
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
                            output.write(response.as_bytes())
                                .and(output.write(MESSAGE_END.as_bytes()))
                                .and(output.flush());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_returns_exit() {
        let iut = Commands {
            monitors: &HashMap::new(),
        };
        assert_eq!(iut.on_command("quit"), either::Right(Exit::Exit));
    }
    
    #[test]
    fn no_monitors() {
        let iut = Commands {
            monitors: &HashMap::new(),
        };
        assert_eq!(iut.on_command("monitors"), either::Left("".to_owned()));
    }

    #[test]
    fn test_monitors_supported() {
        let iut = Commands {
            monitors: &HashMap::new(),
        };
        assert_eq!(iut.on_command("test monitors"), either::Left("1".to_owned()));
    }
}