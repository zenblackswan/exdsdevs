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
    path::PathBuf,
    str::FromStr,
};

use serde_json::Map;

use crate::{
    containers::{Bag, Mail, Value},
    model::Model,
    observer::Observer,
    time::Time,
};

enum LogEvent {
    None,
    Init {
        init_time: Time,
        init_value: Value,
        init_state: Value,
        t_next: Time,
    },
    Outputs {
        sim_time: Time,
        bag: Bag,
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
        mail: Mail,
        elapsed: Time,
    },
    ExternalMailTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        mail: Mail,
        elapsed: Time,
    },
    PreExternalTransition {
        sim_time: Time,
        from_state: Value,
        x_bag: Bag,
        elapsed: Time,
    },
    ExternalTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        x_bag: Bag,
        elapsed: Time,
    },
    PreConfluentTransition {
        sim_time: Time,
        from_state: Value,
        x_bag: Bag,
    },
    ConfluentTransition {
        sim_time: Time,
        from_state: Value,
        to_state: Value,
        t_next: Time,
        x_bag: Bag,
    },
    AfterSubmodelsTransition {
        state: Value,
        sim_time: Time,
        t_next: Time,
    },
}

pub struct Logger {
    log_event: LogEvent,
    stream: Option<BufWriter<File>>,
}

impl Observer for Logger {
    fn new() -> Logger {
        Logger::new()
    }

    fn init_observer(&mut self, config: &Value) {
        let sim_dir = config
            .as_object()
            .unwrap()
            .get("sim_dir")
            .unwrap()
            .as_str()
            .unwrap();
        let model_path = config
            .as_object()
            .unwrap()
            .get("model_full_name")
            .unwrap()
            .as_str()
            .unwrap();
        let mut model_log_file = PathBuf::from_str(sim_dir).unwrap();
        model_log_file.push(model_path);
        let model_log_file = model_log_file.with_extension("log");
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

        self.log_event = LogEvent::None;
        self.stream = Some(BufWriter::new(log_file))
    }

    fn on_init(&mut self, model: &Model, init_time: Time, init_value: &Value, t_next: Time) {
        let init_state = model.state();
        let log_event = LogEvent::Init {
            init_time,
            init_value: init_value.clone(),
            init_state,
            t_next,
        };
        self.write(log_event);
    }

    fn on_outputs(&mut self, _model: &Model, sim_time: Time, bag: &Bag) {
        let log_event = LogEvent::Outputs {
            sim_time,
            bag: bag.to_vec(),
        };
        self.write(log_event);
    }

    fn before_internal_transition(&mut self, model: &Model, sim_time: Time) {
        let from_state = model.state();
        self.log_event = LogEvent::PreInternalTransition {
            sim_time,
            from_state,
        };
    }

    fn after_internal_transition(&mut self, model: &Model, _sim_time: Time, t_next: Time) {
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
        model: &Model,
        sim_time: Time,
        x_bag: &Bag,
        elapsed: Time,
    ) {
        let from_state = model.state();
        self.log_event = LogEvent::PreExternalTransition {
            sim_time,
            from_state,
            x_bag: x_bag.to_vec(),
            elapsed,
        };
    }

    fn after_external_transition(&mut self, model: &Model, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreExternalTransition {
            sim_time,
            from_state,
            x_bag,
            elapsed,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ExternalTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                x_bag,
                elapsed,
            };
            self.write(log_event);
        }
    }

    fn before_external_mail_transition(
        &mut self,
        model: &Model,
        sim_time: Time,
        mail: &Mail,
        elapsed: Time,
    ) {
        let from_state = model.state();
        self.log_event = LogEvent::PreExternalMailTransition {
            sim_time,
            from_state,
            mail: mail.to_vec(),
            elapsed,
        };
    }

    fn after_external_mail_transition(&mut self, model: &Model, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreExternalMailTransition {
            sim_time,
            from_state,
            mail,
            elapsed,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ExternalMailTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                mail,
                elapsed,
            };
            self.write(log_event);
        }
    }

    fn before_confluent_transition(&mut self, model: &Model, sim_time: Time, x_bag: &Bag) {
        let from_state = model.state();
        self.log_event = LogEvent::PreConfluentTransition {
            sim_time,
            from_state,
            x_bag: x_bag.to_vec(),
        };
    }

    fn after_confluent_transition(&mut self, model: &Model, _sim_time: Time, t_next: Time) {
        let log_event = replace(&mut self.log_event, LogEvent::None);
        if let LogEvent::PreConfluentTransition {
            sim_time,
            from_state,
            x_bag,
        } = log_event
        {
            let to_state = model.state();
            let log_event = LogEvent::ConfluentTransition {
                sim_time,
                from_state,
                to_state,
                t_next,
                x_bag,
            };
            self.write(log_event);
        }
    }

    fn after_submodels_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {
        let state = model.state();
        let log_event = LogEvent::AfterSubmodelsTransition {
            state,
            sim_time,
            t_next,
        };
        self.write(log_event);
    }

    fn after_finish(&mut self, _model: &Model, _sim_time: Time) {
        self.flush();
    }

    fn before_finish(&mut self, _model: &Model, _sim_time: Time) {}

    fn result(&self) -> Option<Value> {
        None
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        Self {
            log_event: LogEvent::None,
            stream: None,
        }
    }

    fn write(&mut self, log_event: LogEvent) {
        match log_event {
            LogEvent::Init {
                init_time,
                init_value,
                init_state,
                t_next,
            } => {
                let mut event_map = Map::new();
                event_map.extend([
                    ("TIME".to_owned(), Value::from(&init_time)),
                    ("EVENT".to_owned(), Value::String("INIT".to_owned())),
                    ("INIT_VALUE".to_owned(), init_value),
                    ("INIT_STATE".to_owned(), init_state),
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                ]);
                self.internal_write(&Value::Object(event_map));
            }
            LogEvent::Outputs { sim_time, bag } => {
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
                mail,
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
                x_bag,
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
                x_bag,
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
                    ("TIME_NEXT".to_owned(), Value::from(&t_next)),
                    ("X_BAG".to_owned(), bag_val),
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

    fn get_bag_val(x_bag: Bag) -> Value {
        Value::Array(x_bag.iter().map(Value::from).collect::<Vec<Value>>())
    }

    fn internal_write(&mut self, value: &Value) {
        if let Some(stream) = &mut self.stream {
            let val = serde_json::to_string(value).unwrap();
            stream.write_all(val.as_bytes()).unwrap();
            stream.write_all("\n".as_bytes()).unwrap();
            self.flush();
        }
    }

    pub(crate) fn flush(&mut self) {
        if let Some(stream) = &mut self.stream {
            stream.flush().unwrap();
        }
    }
}
