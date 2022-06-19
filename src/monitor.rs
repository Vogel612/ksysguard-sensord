use crate::config::{  MonitorDefinition, MonitorKind, ValueKind };

use std::process::{ Command, Stdio };
use std::io::{ BufReader, BufRead };

use std::sync::{ RwLock, Arc };

use thread_control::{ Flag };

pub trait Monitor {
    fn meta(&self) -> &SensorMeta;
    fn name(&self) -> &str;
    fn defined_command(&self) -> &str;
    fn get_value(&self) -> String;
    fn start(&self, flag: Arc<Option<Flag>>) -> Option<std::thread::JoinHandle<()>>;
}

pub trait AsOutput {
    fn as_output(&self) -> String;
}

pub fn create(definition: &MonitorDefinition) -> Box<dyn Monitor> {
    let meta = parse_value_meta(definition);

    if definition.kind == MonitorKind::Listening {
        let lm = ListeningMonitor {
            meta: meta,
            name: definition.name.to_owned(),
            defined_command: definition.command.to_owned(),
            current_value: Arc::new(RwLock::new(String::new())),
        };
        return Box::new(lm);
    } else {
        let pm = PollingMonitor {
            meta: meta,
            name: definition.name.to_owned(),
            defined_command: definition.command.to_owned(),
        };
        return Box::new(pm);
    }
}

fn parse_value_meta(definition: &MonitorDefinition) -> SensorMeta {

    if definition.value_kind == ValueKind::Single {
        let mut defaults = SingleValueSensorMeta::default();
        defaults.description = definition.description.to_owned();
        let value_spec: Vec<&str> = definition.value_spec.splitn(3, " ").collect();
        if value_spec.len() < 2 {
            panic!("Invalid value specifier for monitor {}", definition.name);
        }
        // could use unsafe get_unchecked instead of unwrap, but I don't like that very much.
        defaults.min = value_spec.get(0).unwrap().to_string();
        defaults.max = value_spec.get(1).unwrap().to_string();
        defaults.unit = value_spec.get(2).map(|x| x.to_string());
        return SensorMeta::Single(defaults)
    } else {
        // assume ValueKind::Multi
        let value_spec: Vec<(String, MultiValueUnit)> = definition.value_spec.split('\t')
            .map(|v| {
                let len = v.len();
                if len <= 3 {
                    panic!("Invalid value specifier for monitor {}", definition.name);
                }
                (v.get(..(len-2)).unwrap().to_string()
                    , v.get(len-1..).unwrap().into())
            })
            .collect();
        return SensorMeta::Multi(MultiValueSensorMeta {
            fields : value_spec.into_boxed_slice()
        });
    }

}

pub struct PollingMonitor {
    pub name: String,
    pub defined_command: String,
    pub meta: SensorMeta,
}

pub struct ListeningMonitor {
    pub name: String,
    pub defined_command: String,
    pub current_value: Arc<RwLock<String>>,
    pub meta: SensorMeta,
}

pub enum SensorMeta {
    Single(SingleValueSensorMeta),
    Multi(MultiValueSensorMeta)
}

#[derive(Default)]
pub struct SingleValueSensorMeta {
    pub description: String,
    pub min: String,
    pub max: String,
    pub unit: Option<String>,
}

impl AsOutput for SingleValueSensorMeta {
    //  Single value sensors return [ description, min, max, unit ].join("\t")
    //   if min & max are 0, the UI is auto-ranged, the unit is optional (e.g. pscount)
    fn as_output(&self) -> String {
        format!("{}\t{}\t{}\t{}", self.description, self.min, self.max, self.unit.as_ref().unwrap_or(&String::new()))
    }
}

pub enum MultiValueUnit {
    Integer,
    LocalizedInteger,
    Floating,
    String,
    LocalizedString
}

impl From<&str> for MultiValueUnit {
    fn from(v: &str) -> Self { match v.chars().nth(0).unwrap() {
        'd' => MultiValueUnit::Integer,
        'D' => MultiValueUnit::LocalizedInteger,
        'f' => MultiValueUnit::Floating,
        's' => MultiValueUnit::String,
        'S' => MultiValueUnit::LocalizedString,
        _ => panic!("Invalid value type in sensor definition")
    } }
}

impl AsOutput for MultiValueUnit {

    //  Multivalue sensors return a table spec in two lines
    //      -> tab-delimited headers and associated unit spec
    //       d: integer value
    //       D: integer value that should be localized in the frontend
    //       f: floating point value
    //       s: string value
    //       S: string value that needs to be translated
    //          Strings must be added to the ProcessLis::columnDict dictionary.
    fn as_output(&self) -> String {
        match self {
            MultiValueUnit::Integer => "d",
            MultiValueUnit::LocalizedInteger => "D",
            MultiValueUnit::Floating => "f",
            MultiValueUnit::String => "s",
            MultiValueUnit::LocalizedString => "S",
        }.into()
    }
}

pub struct MultiValueSensorMeta {
    pub fields: Box<[(String, MultiValueUnit)]>,
}

impl AsOutput for MultiValueSensorMeta {
    fn as_output(&self) -> String {
        let (mut headers, mut types) = (Vec::new(), Vec::new());
        for (name, unit) in self.fields.iter() {
            headers.push(name.to_owned());
            types.push(unit.as_output());
        }
        return format!("{}\n{}", &headers.join("\t"), types.join("\t"))
    }
}

fn slice_command_spec<'a>(cmd_spec: &'a str) -> (String, Vec<String>) {
    let parts: Vec<String> = cmd_spec.split(" ")
        .map(|c| c.to_owned())
        .collect();
    return (parts[0].to_owned(), parts[1..].to_vec());
}


impl Monitor for PollingMonitor {
    fn meta(&self) -> &SensorMeta { &self.meta }
    fn name(&self) -> &str { &*self.name }
    fn defined_command(&self) -> &str { &*self.defined_command }
    fn get_value(&self) -> String {
        let (program, args) = slice_command_spec(&self.defined_command);
        let output = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .output()
            .expect(&*format!("Could not start subprocess for monitor {}.", self.name));

        return String::from_utf8(output.stdout)
            .expect(&*format!("Child process output for monitor {} was not parseable as UTF-8", self.name));
    }
    fn start(&self, _flag: Arc<Option<Flag>>) -> std::option::Option<std::thread::JoinHandle<()>> {
        return Option::None;
    }
}

impl Monitor for ListeningMonitor {
    fn meta(&self) -> &SensorMeta { &self.meta }
    fn name(&self) -> &str { &*self.name }
    fn defined_command(&self) -> &str { &*self.defined_command }
    fn get_value(&self) -> String {
        let read_guard = self.current_value.read().unwrap();
        return read_guard.to_owned();
    }

    fn start(&self, flag: Arc<Option<Flag>>) -> Option<std::thread::JoinHandle<()>> {
        let (program, args) = slice_command_spec(&self.defined_command);
        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .expect(&*format!("Could not start subprocess for monitor {}.", self.name));
            
        let stdout = child.stdout.take().unwrap();

        let clone = Arc::clone(&self.current_value);

        let handle = std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let buffer = &mut String::new();
            loop {
                // clear buffer because read_line appends
                buffer.clear();
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
                {
                    // explicit block to make me feel better about not being able to explicitly call guard.drop()
                    let mut guard = clone.write().unwrap();
                    *guard = buffer.to_owned();
                    // guard.drop();
                }

                match &*flag {
                    Some(control_flag) => {
                        if !control_flag.is_alive() {
                            break;
                        }
                    },
                    None => {}
                }
            }
        });
        return Option::Some(handle);
    }
}