use configparser::ini::Ini;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    /// we're stealing ksysguardd's port here, not sure how reasonable that is yet.
    pub port: u16,

    /// the default polling interval across sensors
    pub update_interval: u8,
}

impl Config {
    const DEFAULT_PORT: u16 = 3112;
    const DEFAULT_UPDATE_INTERVAL: u8 = 2;

    pub fn read_config(path: & std::path::Path) -> Config {
        let mut config = Ini::new();
        config.load(path).unwrap();
        
        let mut cfg = Config { 
            port: Config::DEFAULT_PORT,
            update_interval: Config::DEFAULT_UPDATE_INTERVAL,
        };
        match config.get("default", "port") {
            Some(port) => cfg.port = u16::from_str(&port).unwrap(),
            None => {}
        }
        match config.get("default", "update interval") {
            Some(interval) => cfg.update_interval = u8::from_str(&interval).unwrap(),
            None => {}
        }
        return cfg;
    }
}
