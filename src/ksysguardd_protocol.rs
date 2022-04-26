use crate::config::{ Config };
use crate::monitor::{ Monitor, create as create_monitor };

use tokio::net::{ TcpListener, TcpStream };
use std::collections::{ HashMap };
use std::io::{ BufReader, BufWriter, BufRead };

const MESSAGE_END: &str = "\nksysguardd> ";
const DEFAULT_BIND_SPEC: &str = "0.0.0.0";

struct Commands {
    // a monitor has multiple sensors?!
    //   (like "nvidia-smi dmon" contains multiple information points)
    // it maintains their metainformation (datatype, value range, ...)
    // for monitoring sensors it retains their latest value
    // for polling sensors it is responsible for getting their value
    pub monitors: HashMap<&'static str, Box<dyn Monitor>>,
}

// required commands:
// monitors, quit
// a description for a sensor
// recommended "commands":
// test, specific sensors involve sensors
impl Commands {
    fn on_command(command: &str, reply_to: fn(&str) -> ()) {
        if command.eq_ignore_ascii_case("quit") {
            // terminate!
        }
        if command.starts_with("test") {
            // checks whether a given command is supported
            // current ksysguardd does not actually support this, soo ...
        }
        if command.starts_with("monitors") {
            // list all monitors
        }
        // attempt to find the monitor specified, give information if terminated by ?
        // return UNKNOWN COMMAND if no matching monitor is found
    }
    // TODO "RECONFIGURE" over stderr??
}

pub struct Sensord {}

impl Sensord {
    pub async fn start(config: &'static Config) {

        let mut known_monitors: HashMap<&str, Box<dyn Monitor>> = HashMap::new();
        for monitor_definition in config.monitors.iter() {
            known_monitors.insert(&*monitor_definition.name, create_monitor(&monitor_definition));
        }

        // establish listening socket
        let listener = TcpListener::bind((DEFAULT_BIND_SPEC, config.port)).await
            .expect("Failed to bind listening port to accept connections.");
        let protocol = Commands {
            monitors: known_monitors,
        };
        loop {
            // let (socket, _) = listener.accept().await?;
        }

        // start sensor data collection (for monitor sensors)
        // register polling sensors
    }
}
