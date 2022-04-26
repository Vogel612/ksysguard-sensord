use crate::config::{  MonitorDefinition, MonitorKind };

use std::process::{ Command, Stdio };

use std::io::{ BufReader, BufRead };

pub trait Monitor {
    fn name(&self) -> &str;
    fn defined_command(&self) -> &str;
    fn get_value(&self) -> String;
}

pub fn create(definition: &MonitorDefinition) -> Box<dyn Monitor> {
    if definition.kind == MonitorKind::Listening {
        let lm = ListeningMonitor {
            name: definition.name.to_owned(),
            current_value: String::new(),
            defined_command: definition.command.to_owned(),
        };
        lm.start();
        return Box::new(lm);
    } else {
        let pm = PollingMonitor {
            name: definition.name.to_owned(),
            defined_command: definition.command.to_owned(),
        };
        return Box::new(pm);
    }
}

pub struct PollingMonitor {
    pub name: String,
    pub defined_command: String,
}

pub struct ListeningMonitor {
    pub name: String,
    pub defined_command: String,
    pub current_value: String,
}

impl ListeningMonitor {
    pub fn start(mut self) -> std::thread::JoinHandle<()> {
        let mut child = Command::new(&*self.defined_command)
            .stdout(Stdio::piped())
            .spawn()
            .expect(&*format!("Could not start subprocess for monitor {}.", self.name));
            
        let stdout = child.stdout.take().unwrap();
        
        let handle = std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let buffer = &mut String::new();
            loop {
                let bytes = reader.read_line(buffer);
                match bytes {
                    Ok(count) if count == 0 => {
                        // count 0 means EOF has been reached
                        break;
                    },
                    Ok(_) => {},
                    Err(_e) => {
                        // not sure what exactly we can do here
                    }
                }
                self.current_value = buffer.to_string();
            }
        });
        return handle;
    }
}

impl Monitor for PollingMonitor {
    fn name(&self) -> &str { todo!() }
    fn defined_command(&self) -> &str { todo!() }
    fn get_value(&self) -> String { 
        let output = Command::new(&*self.defined_command)
            .stdout(Stdio::piped())
            .output()
            .expect(&*format!("Could not start subprocess for monitor {}.", self.name));

        return String::from_utf8(output.stdout)
            .expect(&*format!("Child process output for monitor {} was not parseable as UTF-8", self.name));
    }
}

impl Monitor for ListeningMonitor {
    fn name(&self) -> &str { &*self.name }
    fn defined_command(&self) -> &str { &*self.defined_command }
    fn get_value(&self) -> String { self.current_value.to_owned() }
}