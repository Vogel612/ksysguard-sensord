use crate::config::{ Config, MonitorDefinition, MonitorKind };
use crate::monitor::{ PollingMonitor, ListeningMonitor, Monitor };

use tokio::net::{ TcpListener };
use std::collections::{ hash_map };

const MESSAGE_END: &str = "\nksysguardd> ";
const DEFAULT_BIND_SPEC: &str = "0.0.0.0";

struct Commands {
    // a monitor has multiple sensors.
    //   (like "nvidia-smi dmon" contains multiple information points)
    // it maintains their metainformation (datatype, value range, ...)
    // for monitoring sensors it retains their latest value
    // for polling sensors it is responsible for getting their value
    monitors: String, //Monitors,
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
    pub async fn start(config: &Config) {

        // let monitor_state = 
        let mut known_monitors: hash_map::HashMap<&str, Box<dyn Monitor>> = hash_map::HashMap::new();
        // let mut known_monitors: hash_map::HashMap<&str, ()> = hash_map::HashMap::new();
        for monitor_definition in config.monitors.iter() {
            if monitor_definition.kind == MonitorKind::Listening {
                let lm = ListeningMonitor {
                    name: monitor_definition.name.to_owned(),
                    current_value: String::new(),
                    defined_command: monitor_definition.command.to_owned(),
                };
                known_monitors.insert(&*monitor_definition.name, Box::new(lm));
            } else {
                let pm = PollingMonitor {
                    name: monitor_definition.name.to_owned(),
                    defined_command: monitor_definition.command.to_owned(),
                };
                known_monitors.insert(&*monitor_definition.name, Box::new(pm));
            }
        }


        // establish listening socket
        let listener = TcpListener::bind((DEFAULT_BIND_SPEC, config.port)).await
            .expect("Failed to bind listening port to accept connections.");

        loop {
            // let (socket, _) = listener.accept().await?;
        }

        // start sensor data collection (for monitor sensors)
        // register polling sensors
    }
}
