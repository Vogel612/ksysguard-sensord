
use socket2::Socket;
use crate::config::Config;
// use monitors::{ Monitors, Sensor };

// required commands:
// monitors, quit
// a description for a sensor
// recommended "commands":
// test, specific sensors involve sensors

const MESSAGE_END: &str = "\nksysguardd> ";

struct Commands {
    // a monitor has multiple sensors.
    //   (like "nvidia-smi dmon" contains multiple information points)
    // it maintains their metainformation (datatype, value range, ...)
    // for monitoring sensors it retains their latest value
    // for polling sensors it is responsible for getting their value
    monitors: String //Monitors,
}

impl Commands {
    fn on_command(command: &str, reply_to: Socket) {
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

pub struct Sensord {
    
}

impl Sensord {

    pub fn start(config: &Config) {
        // establish listening socket
        // start sensor data collection (for monitor sensors)
        // register polling sensors 
    }
}
