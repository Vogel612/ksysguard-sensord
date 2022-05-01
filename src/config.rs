use configparser::ini::Ini;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    /// we're stealing ksysguardd's port here, not sure how reasonable that is yet.
    pub port: u16,

    /// the default polling interval across sensors
    pub update_interval: u8,

    pub monitors: Vec<MonitorDefinition>,
}

#[derive(Debug)]
pub struct MonitorDefinition {
    pub name: String,
    pub command: String,
    pub kind: MonitorKind,
}

#[derive(Debug, PartialEq)]
pub enum MonitorKind {
    Polling,
    Listening
}

fn get_sensor_kind(cfg_value: &str) -> MonitorKind {
    if cfg_value.eq_ignore_ascii_case("listening") {
        return MonitorKind::Listening;
    } else {
        // default to polling
        return MonitorKind::Polling;
    }
}

impl Config {
    const DEFAULT_PORT: u16 = 3112;
    const DEFAULT_UPDATE_INTERVAL: u8 = 2;

    pub fn read_config(path: &std::path::Path) -> Config {
        let mut config = Ini::new();
        config.load(path).unwrap();

        let mut cfg = Config {
            port: Config::DEFAULT_PORT,
            update_interval: Config::DEFAULT_UPDATE_INTERVAL,
            monitors: Vec::new(),
        };
        match config.get("default", "port") {
            Some(port) => cfg.port = u16::from_str(&port).unwrap(),
            None => {}
        }
        match config.get("default", "update interval") {
            Some(interval) => cfg.update_interval = u8::from_str(&interval).unwrap(),
            None => {}
        }

        for section in config.sections() {
            // section name is sensor name
            match (config.get(&section, "command"), config.get(&section, "kind")) {
                (Some(cmd), Some(kind)) => {
                    cfg.monitors.push(MonitorDefinition{
                        name: section.to_owned(),
                        command: cmd.to_owned(),
                        kind: get_sensor_kind(&kind)
                    });
                },
                (Some(cmd), None) => {
                    cfg.monitors.push(MonitorDefinition{
                        name: section.to_owned(),
                        command: cmd.to_owned(),
                        kind: MonitorKind::Polling
                    })
                },
                _ => {
                    println!("Invalid sensor definition for sensor {}. No command was specified.", section);
                }
            }
        }
        return cfg;
    }
}
