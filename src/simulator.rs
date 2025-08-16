// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

/// Simulation engine module
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    ops::Deref,
    rc::Rc,
};

use rand::rngs::StdRng;

use crate::{
    containers::{Bag, Mail, MailItem, ModelSimResults, Msg},
    model::Model,
    observer::Observer,
    sim_model::SimModel,
    time::Time,
};

/// DEVS simulation executor
/// # Execution Lifecycle
/// 1. Initialization
/// 2. Output collection
/// 3. Message processing
/// 4. State transitions
#[derive(Clone, Debug)]
pub struct Simulator {
    pub sim_model: SimModel,
    pub rng: Rc<RefCell<StdRng>>,
    pub imminent: HashSet<String>,
    pub mail: Mail,
    pub t_last: Time,
    pub t_next_self: Time,
    pub t_next: Time,
    pub observers: BTreeMap<String, Box<dyn Observer>>,
    pub iteration: u64,
}

impl Simulator {
    pub fn new(
        model_full_name: String,
        model: Model,
        rng: Rc<RefCell<StdRng>>,
        iteration: u64,
    ) -> Self {
        let observers = model.observers.clone();
        let sim_model = SimModel::new(model_full_name, model, &rng, iteration);
        Simulator {
            sim_model,
            rng,
            imminent: Default::default(),
            mail: Default::default(),
            t_last: Time::Value(0),
            t_next_self: Time::Inf,
            t_next: Time::Inf,
            observers,
            iteration,
        }
    }

    /// Initializes the simulator
    pub(crate) fn init(&mut self, init_time: Time, rng: Rc<RefCell<StdRng>>) {
        self.sim_model
            .init(init_time, &mut self.rng.deref().borrow_mut());

        self.t_last = init_time;
        self.t_next_self = self.t_last
            + self
                .sim_model
                .time_advance(&mut self.rng.deref().borrow_mut());
        self.t_next = self
            .sim_model
            .sub_simulators()
            .and_then(|sub_simulators| {
                Some(sub_simulators.iter_mut().map(|(_, sub_simulator)| {
                    sub_simulator.init(init_time, rng.clone());
                    sub_simulator.t_next()
                }))
            })
            .and_then(|times| times.min())
            .unwrap_or(Time::Inf)
            .min(self.t_next_self);

        for obs in self.observers.values_mut() {
            obs.init(&self.sim_model, self.iteration);
            obs.on_init(&self.sim_model, init_time, self.t_next);
        }

        self.rng = rng;
    }

    pub(crate) fn t_next(&self) -> Time {
        self.t_next
    }

    /// Collects output messages
    pub(crate) fn collect_outputs(&mut self, sim_time: Time) -> Bag {
        let bag = if sim_time == self.t_next_self {
            self.sim_model.output(sim_time)
        } else if sim_time == self.t_next {
            if let Some(subsimulators) = self.sim_model.sub_simulators() {
                for (model_name, simulator) in subsimulators {
                    if simulator.t_next() == sim_time {
                        self.imminent.insert(model_name.clone());
                        let y_bag = simulator.collect_outputs(sim_time);
                        self.mail.push(MailItem {
                            model_name: model_name.clone(),
                            y_bag,
                        });
                    }
                }
            }
            self.sim_model.get_y_bag_from_mail(&self.mail)
        } else {
            panic!("DEVS ERROR: Bad synchronization in Simulator.collect_outputs()")
        };

        for obs in self.observers.values_mut() {
            obs.on_outputs(&self.sim_model, sim_time, &bag);
        }
        bag
    }

    /// Processes inter-model messages
    pub(crate) fn process_y_messages(&mut self, sim_time: Time) {
        let elapsed = sim_time - self.t_last;
        self.t_last = sim_time;

        for obs in self.observers.values_mut() {
            obs.before_external_mail_transition(&self.sim_model, sim_time, &self.mail, elapsed);
        }

        self.sim_model.external_mail_transition(
            sim_time,
            elapsed,
            &self.mail,
            &mut self.rng.deref().borrow_mut(),
        );
        // warning!: The following two lines of code may violate the exdevs formalism.
        // This must be taken into account when obtaining erroneous simulation results.
        self.t_next_self = self.t_last
            + self
                .sim_model
                .time_advance(&mut self.rng.deref().borrow_mut());
        self.t_next = self.t_next_self.min(self.t_next);

        for obs in self.observers.values_mut() {
            obs.after_external_mail_transition(&self.sim_model, sim_time, self.t_next_self);
        }
    }

    pub(crate) fn process_x_messages(&mut self, sim_time: Time, x_bag: Bag) {
        if sim_time >= self.t_last && sim_time <= self.t_next_self {
            let elapsed = sim_time - self.t_last;
            self.t_last = sim_time;

            if sim_time == self.t_next_self {
                // internal transition
                if x_bag.is_empty() {
                    for obs in self.observers.values_mut() {
                        obs.before_internal_transition(&self.sim_model, sim_time);
                    }

                    self.sim_model
                        .internal_transition(sim_time, &mut self.rng.deref().borrow_mut());
                    self.t_next_self = self.t_last
                        + self
                            .sim_model
                            .time_advance(&mut self.rng.deref().borrow_mut());

                    for obs in self.observers.values_mut() {
                        obs.after_internal_transition(&self.sim_model, sim_time, self.t_next_self);
                    }
                // confluent transition
                } else {
                    for obs in self.observers.values_mut() {
                        obs.before_confluent_transition(&self.sim_model, sim_time, &x_bag);
                    }

                    self.sim_model.confluent_transition(
                        sim_time,
                        &x_bag,
                        &mut self.rng.deref().borrow_mut(),
                    );
                    self.t_next_self = self.t_last
                        + self
                            .sim_model
                            .time_advance(&mut self.rng.deref().borrow_mut());

                    for obs in self.observers.values_mut() {
                        obs.after_confluent_transition(&self.sim_model, sim_time, self.t_next_self);
                    }
                }

            // external transition
            } else {
                for obs in self.observers.values_mut() {
                    obs.before_external_transition(&self.sim_model, sim_time, &x_bag, elapsed);
                }

                self.sim_model.external_transition(
                    sim_time,
                    elapsed,
                    &x_bag,
                    &mut self.rng.deref().borrow_mut(),
                );
                self.t_next_self = self.t_last
                    + self
                        .sim_model
                        .time_advance(&mut self.rng.deref().borrow_mut());

                for obs in self.observers.values_mut() {
                    obs.after_external_transition(&self.sim_model, sim_time, self.t_next_self);
                }
            }

            if self.has_submodels() {
                self.sent_x_bag_to_submodels(sim_time, &x_bag);
                let submodels_t_next = self.submodels_t_next();
                self.t_next = self.t_next_self.min(submodels_t_next);

                for obs in self.observers.values_mut() {
                    obs.after_submodels_transition(&self.sim_model, sim_time, self.t_next);
                }
            } else {
                self.t_next = self.t_next_self;
            }
        } else {
            panic!("Bad synchronization in Simulator::process_x_messages()")
        }
    }

    fn submodels_t_next(&mut self) -> Time {
        self.sim_model
            .sub_simulators()
            .and_then(|sub_simulators| {
                sub_simulators
                    .iter_mut()
                    .map(|(_, subsim)| subsim.t_next())
                    .min()
            })
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
            self.sim_model
                .get_subsimulator(simulator_name)
                .iter_mut()
                .for_each(|simulator| {
                    simulator.process_x_messages(sim_time, Bag::new());
                });
        }

        self.imminent.clear();

        x_bags_for_submodels
            .into_iter()
            .for_each(|(simulator_name, tmp_x_bag)| {
                self.sim_model
                    .get_subsimulator(&simulator_name)
                    .and_then(|simulator| Some(simulator.process_x_messages(sim_time, tmp_x_bag)));
            });
    }

    fn get_submodels_x_bags(&self, x_bag: &Bag) -> BTreeMap<String, Bag> {
        let eic = self.sim_model.external_input_couplings().unwrap();
        let ic = self.sim_model.internal_couplings().unwrap();

        let mut x_bags_for_submodels: BTreeMap<String, Bag> = BTreeMap::new();

        for Msg { port, value } in x_bag {
            for (model, ports) in &eic[port] {
                x_bags_for_submodels
                    .entry(model.clone())
                    .or_default()
                    .extend(ports.iter().map(|port| Msg {
                        port: port.clone(),
                        value: value.clone(),
                    }));
            }
        }

        for MailItem { model_name, y_bag } in &self.mail {
            for Msg { port, value } in y_bag {
                if let Some(mdl) = ic.get(model_name) {
                    if let Some(mdl_prts) = mdl.get(port) {
                        for (model, ports) in mdl_prts {
                            x_bags_for_submodels
                                .entry(model.clone())
                                .or_default()
                                .extend(ports.iter().map(|port| Msg {
                                    port: port.clone(),
                                    value: value.clone(),
                                }));
                        }
                    }
                }
            }
        }
        x_bags_for_submodels
    }

    fn has_submodels(&self) -> bool {
        self.sim_model.has_submodels()
    }

    fn sim_model_full_name(&self) -> &str {
        self.sim_model.full_name()
    }

    pub(crate) fn finish(&mut self, sim_time: Time) -> BTreeMap<String, ModelSimResults> {
        let mut results = self.sim_model.finish(sim_time);
        let mut obs_results: ModelSimResults = ModelSimResults::new();
        for (obs_name, obs) in self.observers.iter_mut() {
            if let Some(obs_result) = obs.finish(&self.sim_model, sim_time) {
                obs_results.insert(obs_name.clone(), obs_result);
            };
        }
        if !obs_results.is_empty() {
            results.insert(self.sim_model_full_name().to_owned(), obs_results);
        }
        results
    }
}
