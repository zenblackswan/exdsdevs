// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    fs::{DirBuilder, File, OpenOptions},
    io::{BufWriter, Write},
    mem::replace,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde_json::Map;

use crate::{
    containers::{Bag, Mail, MailItem, Msg, Value},
    observer::Observer,
    sim_model::SimModel,
    time::Time,
};

#[derive(Clone)]
struct LogMsg {
    port: String,
    value: Value,
}

impl From<&Msg> for LogMsg {
    fn from(msg: &Msg) -> Self {
        LogMsg {
            port: msg.port.clone(),
            value: msg.value.deref().clone(),
        }
    }
}

type LogBag = Vec<LogMsg>;

fn log_bag_from_bag(bag: &Bag) -> LogBag {
    bag.iter().map(From::from).collect()
}

#[derive(Clone)]
pub struct LogMailItem {
    model_name: String,
    y_bag: LogBag,
}

impl From<&MailItem> for LogMailItem {
    fn from(mail_item: &MailItem) -> Self {
        LogMailItem {
            model_name: mail_item.model_name.clone(),
            y_bag: mail_item.y_bag.iter().map(From::from).collect(),
        }
    }
}

type LogMail = Vec<LogMailItem>;

fn log_mail_from_mail(mail: &Mail) -> LogMail {
    mail.iter().map(From::from).collect()
}

#[derive(Clone)]
enum LogEvent {
    None,
    Init {
        init_time: Time,
        init_state: Value,
        t_next: Time,
    },
    Outputs {
        sim_time: Time,
        log_bag: LogBag,
    },
    PreInternalTransition {
        sim_time: Time,
        from_state: Value,
    },
    InternalTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
    },
    PreExternalMailTransition {
        sim_time: Time,
        from_state: Value,
        log_mail: LogMail,
        elapsed: Time,
    },
    ExternalMailTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        log_mail: LogMail,
        elapsed: Time,
    },
    PreExternalTransition {
        sim_time: Time,
        from_state: Value,
        log_x_bag: LogBag,
        elapsed: Time,
    },
    ExternalTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        log_x_bag: LogBag,
        elapsed: Time,
    },
    PreConfluentTransition {
        sim_time: Time,
        from_state: Value,
        log_x_bag: LogBag,
    },
    ConfluentTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        log_x_bag: LogBag,
    },
    AfterSubmodelsTransition {
        state: Value,
        sim_time: Time,
        t_next: Time,
    },
}

#[derive(Clone)]
pub struct Logger {
    log_event: LogEvent,
    out_dir: PathBuf,
    log_file: Option<Arc<Mutex<BufWriter<File>>>>,
}

impl Observer for Logger {
    fn init(&mut self, model: &SimModel, iteration: u64) {
        let model_log_file = self
            .out_dir
            .join(format!("iter_{iteration}"))
            .join(&model.full_name)
            .with_extension("log");
        let model_log_dir = model_log_file.parent().unwrap();

        if !model_log_dir.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(model_log_dir)
                .unwrap()
        }
        let log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(model_log_file)
            .unwrap();

        self.log_file = Some(Arc::new(Mutex::new(BufWriter::new(log_file))));
    }

    fn on_init(&mut self, model: &SimModel, init_time: Time, t_next: Time) {
        let init_state = model.state();
        let log_event = LogEvent::Init {
            init_time,
            init_state,
            t_next,
        };
        self.write(log_event);
    }

    fn on_outputs(&mut self, _model: &SimModel, sim_time: Time, bag: &Bag) {
        let log_event = LogEvent::Outputs {
            sim_time,
            log_bag: log_bag_from_bag(bag),
        };
        self.write(log_event);
    }

    fn before_internal_transition(&mut self, model: &SimModel, sim_time: Time) {
        let from_state = model.state();
        self.log_event = LogEvent::PreInternalTransition {
            sim_time,
            from_state,
        };
    }

    fn after_internal_transition(&mut self, model: &SimModel, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreInternalTransition {
            sim_time,
            from_state,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::InternalTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
            };
            self.write(log_event);
        }
    }

    fn before_external_transition(
        &mut self,
        model: &SimModel,
        sim_time: Time,
        x_bag: &Bag,
        elapsed: Time,
    ) {
        let from_state = model.state();
        self.log_event = LogEvent::PreExternalTransition {
            sim_time,
            from_state,
            log_x_bag: log_bag_from_bag(x_bag),
            elapsed,
        };
    }

    fn after_external_transition(&mut self, model: &SimModel, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreExternalTransition {
            sim_time,
            from_state,
            log_x_bag,
            elapsed,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ExternalTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_x_bag,
                elapsed,
            };
            self.write(log_event);
        }
    }

    fn before_external_mail_transition(
        &mut self,
        model: &SimModel,
        sim_time: Time,
        mail: &Mail,
        elapsed: Time,
    ) {
        let from_state = model.state();
        self.log_event = LogEvent::PreExternalMailTransition {
            sim_time,
            from_state,
            log_mail: log_mail_from_mail(mail),
            elapsed,
        };
    }

    fn after_external_mail_transition(&mut self, model: &SimModel, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreExternalMailTransition {
            sim_time,
            from_state,
            log_mail,
            elapsed,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ExternalMailTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_mail,
                elapsed,
            };
            self.write(log_event);
        }
    }

    fn before_confluent_transition(&mut self, model: &SimModel, sim_time: Time, x_bag: &Bag) {
        let from_state = model.state();
        self.log_event = LogEvent::PreConfluentTransition {
            sim_time,
            from_state,
            log_x_bag: log_bag_from_bag(x_bag),
        };
    }

    fn after_confluent_transition(&mut self, model: &SimModel, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreConfluentTransition {
            sim_time,
            from_state,
            log_x_bag,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ConfluentTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_x_bag,
            };
            self.write(log_event);
        }
    }

    fn after_submodels_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {
        let state = model.state();
        let log_event = LogEvent::AfterSubmodelsTransition {
            state,
            sim_time,
            t_next,
        };
        self.write(log_event);
    }
}

impl Logger {
    pub fn new(out_dir: &Path) -> Logger {
        Self {
            log_event: LogEvent::None,
            out_dir: out_dir.to_owned(),
            log_file: None,
        }
    }

    fn write(&mut self, log_event: LogEvent) {
        match log_event {
            LogEvent::Init {
                init_time,
                init_state,
                t_next,
            } => {
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&init_time)),
                    ("EVENT".to_owned(), Value::String("INIT".to_owned())),
                    ("INIT_STATE".to_owned(), init_state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::Outputs {
                sim_time,
                log_bag: bag,
            } => {
                let bag_val = Self::get_bag_val(bag);
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    ("EVENT".to_owned(), Value::String("OUTPUTS".to_owned())),
                    ("BAG".to_owned(), bag_val),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::InternalTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
            } => {
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    (
                        "EVENT".to_owned(),
                        Value::String("INTERNAL_TRANSITION".to_owned()),
                    ),
                    ("FROM".to_owned(), from_state),
                    ("TO".to_owned(), to_state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::ExternalMailTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_mail: mail,
                elapsed,
            } => {
                let mail_val = Value::Array(mail.iter().map(Value::from).collect::<Vec<Value>>());
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    (
                        "EVENT".to_owned(),
                        Value::String("EXTERNAL_MAIL_TRANSITION".to_owned()),
                    ),
                    ("FROM".to_owned(), from_state),
                    ("TO".to_owned(), to_state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                    ("MAIL".to_owned(), mail_val),
                    ("ELAPSED".to_owned(), Value::from(&elapsed)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::ExternalTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_x_bag: x_bag,
                elapsed,
            } => {
                let bag_val = Self::get_bag_val(x_bag);
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    (
                        "EVENT".to_owned(),
                        Value::String("EXTERNAL_TRANSITION".to_owned()),
                    ),
                    ("FROM".to_owned(), from_state),
                    ("TO".to_owned(), to_state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                    ("X_BAG".to_owned(), bag_val),
                    ("ELAPSED".to_owned(), Value::from(&elapsed)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::ConfluentTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                log_x_bag: x_bag,
            } => {
                let bag_val = Self::get_bag_val(x_bag);
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    (
                        "EVENT".to_owned(),
                        Value::String("CONFLUENT_TRANSITION".to_owned()),
                    ),
                    ("FROM".to_owned(), from_state),
                    ("TO".to_owned(), to_state),
                    ("X_BAG".to_owned(), bag_val),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::AfterSubmodelsTransition {
                state,
                sim_time,
                t_next,
            } => {
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&sim_time)),
                    (
                        "EVENT".to_owned(),
                        Value::String("AFTER_SUBMODELS_TRANSITION".to_owned()),
                    ),
                    ("STATE".to_owned(), state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            _ => {}
        }
    }

    fn get_bag_val(x_bag: LogBag) -> Value {
        Value::Array(x_bag.iter().map(Value::from).collect::<Vec<Value>>())
    }

    fn internal_write(&mut self, value: &Value) {
        if let Some(stream) = &mut self.log_file {
            if let Ok(mut stream) = stream.lock() {
                let val = serde_json::to_string(value).unwrap();
                stream.write_all(val.as_bytes()).unwrap();

                #[cfg(target_os = "linux")]
                stream.write_all("\n".as_bytes()).unwrap();

                #[cfg(target_os = "windows")]
                stream.write_all("\r\n".as_bytes()).unwrap();

                stream.flush().unwrap();
            }
        }
    }
}

impl From<&LogMailItem> for Value {
    fn from(log_mail_item: &LogMailItem) -> Self {
        let msg_vec: Vec<Value> = log_mail_item.y_bag.iter().map(Value::from).collect();
        let mut bag_map = Map::new();
        bag_map.insert(log_mail_item.model_name.clone(), Value::Array(msg_vec));
        Value::Object(bag_map)
    }
}

impl From<&LogMsg> for Value {
    fn from(log_msg: &LogMsg) -> Self {
        let mut val_map = Map::new();
        val_map.insert("PORT".to_owned(), Value::String(log_msg.port.to_owned()));
        val_map.insert("VALUE".to_owned(), log_msg.value.to_owned());
        Value::Object(val_map)
    }
}
