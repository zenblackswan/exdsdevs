// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::containers::{Bag, ModelSimResults};
use crate::errors::ExdsdevsError;

use crate::model::Model;
use crate::{simulator::Simulator, time::Time};

#[derive(Clone)]
pub struct RootSimulator {
    pub simulator: Simulator,
    pub init_time: Time,
    pub finish_time: Time,
    pub sim_time: Time,
}

impl RootSimulator {
    pub fn new(model: Model, iteration: u64) -> Result<RootSimulator, ExdsdevsError> {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));
        let simulator = Simulator::new(String::from("root"), model, rng, iteration);
        let init_time = Time::Value(0);
        let finish_time = Time::Inf;
        let sim_time = init_time;
        Ok(RootSimulator {
            simulator,
            init_time,
            finish_time,
            sim_time,
        })
    }

    pub fn init(&mut self, init_time: Time, finish_time: Time, random_seed: u64) {
        self.init_time = init_time;
        self.finish_time = finish_time;
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(random_seed)));
        self.simulator.init(self.init_time, rng);
        self.sim_time = self.simulator.t_next();
    }

    fn collect_outputs(&mut self) {
        self.simulator.collect_outputs(self.sim_time);
    }

    fn process_y_messages(&mut self) {
        self.simulator.process_y_messages(self.sim_time);
    }

    fn process_x_messages(&mut self) {
        let x_bag = Bag::new();
        self.simulator.process_x_messages(self.sim_time, x_bag);
    }

    fn finish(&mut self, sim_time: Time) -> BTreeMap<String, ModelSimResults> {
        self.simulator.finish(sim_time)
    }

    fn step(&mut self) {
        self.collect_outputs();
        self.process_y_messages();
        self.process_x_messages();
    }

    pub fn run(&mut self) -> BTreeMap<String, ModelSimResults> {
        while self.sim_time < self.finish_time {
            self.step();
            self.sim_time = self.simulator.t_next();
        }
        self.finish(self.sim_time)
    }
}
