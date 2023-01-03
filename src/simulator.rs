// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    rc::Rc,
};

use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::Map;

use crate::{
    containers::{Bag, Mail, MailItem, Msg, Value},
    model::{ExternalInputCoupling, InternalCoupling, Model, Resources},
    observer::Observer,
    time::Time,
};

pub struct Simulator {
    pub full_name: String,
    pub model: Model,
    pub init_value: Value,
    pub rng: Rc<RefCell<StdRng>>,
    pub resources: Resources,
    pub imminent: HashSet<String>,
    pub mail: Mail,
    pub t_last: Time,
    pub t_next_self: Time,
    pub t_next: Time,
    pub sim_dir: PathBuf,
    pub observers: Vec<Box<dyn Observer>>,
}

impl Simulator {
    pub fn new(full_name: &str, model: Model, resources: Resources) -> Self {
        Simulator {
            full_name: full_name.to_owned(),
            model,
            init_value: Value::Null,
            rng: Rc::new(RefCell::new(StdRng::seed_from_u64(0))),
            resources,
            imminent: Default::default(),
            mail: Default::default(),
            t_last: Time::Value(0),
            t_next_self: Time::Inf,
            t_next: Time::Inf,
            sim_dir: Default::default(),
            observers: Default::default(),
        }
    }

    pub(crate) fn init_static(
        &mut self,
        model_full_name: &str,
        sim_dir: &PathBuf,
        init_variant: &BTreeMap<String, Value>,
        rng: Rc<RefCell<StdRng>>,
    ) {
        self.sim_dir = sim_dir.to_owned();
        self.init_value = init_variant.get(model_full_name).unwrap().clone();
        self.rng = rng;
        for (sub_simulator_name, sub_simulator) in self.model.sub_simulators() {
            let sub_simulator_full_name = format!("{}/{}", model_full_name, sub_simulator_name);
            sub_simulator.init_static(
                &sub_simulator_full_name,
                sim_dir,
                init_variant,
                self.rng.clone(),
            );
        }
        let mut observer_config = Value::Object(Map::new());
        observer_config.as_object_mut().unwrap().extend([
            (
                "model_full_name".to_owned(),
                Value::String(model_full_name.to_owned()),
            ),
            (
                "sim_dir".to_owned(),
                Value::String(sim_dir.to_str().unwrap().to_string()),
            ),
        ]);

        for observer in self.observers.iter_mut() {
            observer.init_observer(&observer_config);
        }
    }

    pub(crate) fn init(&mut self, init_time: Time) {
        self.model.init(
            init_time,
            &self.init_value,
            &self.resources,
            &mut self.rng.borrow_mut(),
        );

        self.t_last = init_time;
        self.t_next_self = self.t_last + self.model.time_advance(&mut self.rng.borrow_mut());
        self.t_next = self
            .model
            .sub_simulators()
            .map(|(_, sub_simulator)| {
                sub_simulator.init(init_time);
                sub_simulator.t_next()
            })
            .min()
            .unwrap_or(Time::Inf)
            .min(self.t_next_self);

        for observer in self.observers.iter_mut() {
            observer.on_init(&self.model, init_time, &self.init_value, self.t_next)
        }
    }

    pub fn with_observer(mut self, observer: Box<dyn Observer>) -> Self {
        self.add_observer(observer);
        self
    }

    pub fn add_observer(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    pub(crate) fn t_next(&self) -> Time {
        self.t_next
    }

    pub(crate) fn collect_outputs(&mut self, sim_time: Time) -> Bag {
        let bag = if sim_time == self.t_next_self {
            self.model.output(sim_time)
        } else if sim_time == self.t_next {
            for (model_name, simulator) in self.model.sub_simulators() {
                if simulator.t_next() == sim_time {
                    self.imminent.insert(model_name.clone());
                    let y_bag = simulator.collect_outputs(sim_time);
                    self.mail.push(MailItem {
                        model_name: model_name.clone(),
                        y_bag,
                    });
                }
            }
            self.model.get_y_bag_from_mail(&self.mail)
        } else {
            panic!("DEVS ERROR: Bad synchronization in Simulator.collect_outputs()")
        };

        for observer in self.observers.iter_mut() {
            observer.on_outputs(&self.model, sim_time, &bag)
        }
        bag
    }

    pub(crate) fn process_y_messages(&mut self, sim_time: Time) {
        let elapsed = sim_time - self.t_last;
        self.t_last = sim_time;
        for observer in self.observers.iter_mut() {
            observer.before_external_mail_transition(&self.model, sim_time, &self.mail, elapsed)
        }
        self.model.external_mail_transition(
            sim_time,
            elapsed,
            &self.mail,
            &mut self.rng.borrow_mut(),
        );
        // warning!: The following two lines of code may violate the exdevs formalism.
        // This must be taken into account when obtaining erroneous simulation results.
        self.t_next_self = self.t_last + self.model.time_advance(&mut self.rng.borrow_mut());
        self.t_next = self.t_next_self.min(self.t_next);

        for observer in self.observers.iter_mut() {
            observer.after_external_mail_transition(&self.model, sim_time, self.t_next_self)
        }
    }

    pub(crate) fn process_x_messages(&mut self, sim_time: Time, x_bag: Bag) {
        if sim_time >= self.t_last && sim_time <= self.t_next_self {
            let elapsed = sim_time - self.t_last;
            self.t_last = sim_time;

            if sim_time == self.t_next_self {
                // internal transition
                if x_bag.is_empty() {
                    for observer in self.observers.iter_mut() {
                        observer.before_internal_transition(&self.model, sim_time);
                    }
                    self.model
                        .internal_transition(sim_time, &mut self.rng.borrow_mut());
                    self.t_next_self =
                        self.t_last + self.model.time_advance(&mut self.rng.borrow_mut());

                    for observer in self.observers.iter_mut() {
                        observer.after_internal_transition(&self.model, sim_time, self.t_next_self);
                    }

                // confluent transition
                } else {
                    for observer in self.observers.iter_mut() {
                        observer.before_confluent_transition(&self.model, sim_time, &x_bag);
                    }

                    self.model
                        .confluent_transition(sim_time, &x_bag, &mut self.rng.borrow_mut());
                    self.t_next_self =
                        self.t_last + self.model.time_advance(&mut self.rng.borrow_mut());

                    for observer in self.observers.iter_mut() {
                        observer.after_confluent_transition(
                            &self.model,
                            sim_time,
                            self.t_next_self,
                        );
                    }
                }

            // external transition
            } else {
                for observer in self.observers.iter_mut() {
                    observer.before_external_transition(&self.model, sim_time, &x_bag, elapsed);
                }
                self.model.external_transition(
                    sim_time,
                    elapsed,
                    &x_bag,
                    &mut self.rng.borrow_mut(),
                );
                self.t_next_self =
                    self.t_last + self.model.time_advance(&mut self.rng.borrow_mut());

                for observer in self.observers.iter_mut() {
                    observer.after_external_transition(&self.model, sim_time, self.t_next_self);
                }
            }

            if self.has_submodels() {
                self.sent_x_bag_to_submodels(sim_time, &x_bag);
                let submodels_t_next = self.submodels_t_next();
                self.t_next = self.t_next_self.min(submodels_t_next);
                for observer in self.observers.iter_mut() {
                    observer.after_submodels_transition(&self.model, sim_time, self.t_next);
                }
            } else {
                self.t_next = self.t_next_self;
            }
        } else {
            panic!("Bad synchronization in Simulator::process_x_messages()")
        }
    }

    fn submodels_t_next(&mut self) -> Time {
        self.model
            .sub_simulators()
            .map(|(_, subsim)| subsim.t_next())
            .min()
            .unwrap_or(Time::Inf)
    }

    fn sent_x_bag_to_submodels(&mut self, sim_time: Time, x_bag: &Bag) {
        let x_bags_for_submodels = self.get_submodels_x_bags(x_bag);
        self.mail.clear();

        let has_x_bags: HashSet<String> = x_bags_for_submodels
            .keys()
            .cloned()
            .collect::<HashSet<String>>();
        for simulator_name in self.imminent.difference(&has_x_bags) {
            let simulator = self.model.get_subsimulators(simulator_name);
            simulator.process_x_messages(sim_time, Bag::new());
        }
        self.imminent.clear();

        for (model_name, tmp_x_bag) in x_bags_for_submodels.into_iter() {
            let simulator = self.model.get_subsimulators(&model_name);
            simulator.process_x_messages(sim_time, tmp_x_bag);
        }
    }

    fn get_submodels_x_bags(&self, x_bag: &Bag) -> BTreeMap<String, Bag> {
        let mut x_bags_for_submodels: BTreeMap<String, Bag> = BTreeMap::new();
        for ExternalInputCoupling {
            source_port,
            destination_model,
            destination_model_port,
        } in self.model.external_input_couplings()
        {
            for Msg { port, value } in x_bag.iter() {
                if source_port == port {
                    let tmp_bag = x_bags_for_submodels
                        .entry(destination_model.clone())
                        .or_default();
                    tmp_bag.push(Msg {
                        port: destination_model_port.clone(),
                        value: value.clone(),
                    });
                }
            }
        }

        for InternalCoupling {
            source_model,
            source_model_port,
            destination_model,
            destination_model_port,
        } in self.model.internal_couplings()
        {
            for MailItem { model_name, y_bag } in self.mail.iter() {
                if source_model == model_name {
                    for Msg { port, value } in y_bag.iter() {
                        if port == source_model_port {
                            x_bags_for_submodels
                                .entry(destination_model.clone())
                                .or_default()
                                .push(Msg {
                                    port: destination_model_port.clone(),
                                    value: value.clone(),
                                });
                        }
                    }
                }
            }
        }
        x_bags_for_submodels
    }

    fn has_submodels(&self) -> bool {
        self.model.has_submodels()
    }

    pub(crate) fn finish(&mut self, sim_time: Time) {
        for (_, model) in self.model.sub_simulators() {
            model.finish(sim_time);
        }
        self.model.finish(sim_time);
        for observer in self.observers.iter_mut() {
            observer.after_finish(&self.model, sim_time);
        }
    }
}
