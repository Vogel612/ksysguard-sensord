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
    pub description: String,
    pub value_kind: ValueKind,
    pub value_spec: String
}

#[derive(Debug, PartialEq)]
pub enum MonitorKind {
    Polling,
    Listening
}

#[derive(Debug, PartialEq)]
pub enum ValueKind {
    Single,
    Multi
}

fn get_sensor_kind(cfg_value: &str) -> MonitorKind {
    if cfg_value.eq_ignore_ascii_case("listening") {
        return MonitorKind::Listening;
    } else {
        // default to polling
        return MonitorKind::Polling;
    }
}

fn get_value_kind(cfg_value: &str) -> ValueKind {
    if cfg_value.eq_ignore_ascii_case("multi") {
        return ValueKind::Multi;
    } else {
        // assume Single values from the monitor
        return ValueKind::Single;
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
            if section.eq_ignore_ascii_case("default") {
                continue;
            }
            // section name is sensor name
            let name = section.to_owned();
            let command = config.get(&section, "command")
                .expect(&format!("Invalid sensor definition for sensor {}. No command was specified.", section))
                .to_owned();
            let kind = config.get(&section, "kind")
                .map(|x| get_sensor_kind(&x))
                .unwrap_or(MonitorKind::Polling);
            let description = config.get(&section, "description")
                .unwrap_or("".into());
            let value_kind = config.get(&section, "value_kind")
                .map(|x| get_value_kind(&x))
                .unwrap_or(ValueKind::Single);
            let value_definition = config.get(&section, "value_definition")
                .unwrap_or("0 0 ?".into());

            cfg.monitors.push(MonitorDefinition{
                name: name,
                command: command,
                kind: kind,
                description: description,
                value_kind: value_kind,
                value_spec: value_definition,
            });
        }
        return cfg;
    }
}
