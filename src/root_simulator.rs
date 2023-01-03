// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::containers::{Bag, Value};
use crate::model::ModelFactory;

use crate::{simulator::Simulator, time::Time};

pub struct RootSimulator {
    pub root_model_full_name: String,
    pub simulator: Simulator,
    pub init_time: Time,
    pub finish_time: Time,
    pub sim_time: Time,
    pub rng: Rc<RefCell<StdRng>>,
}

impl RootSimulator {
    pub fn new(
        model_factory: Arc<ModelFactory>,
        root_model_class_name: String,
        root_model_full_name: String,
        global_resources: Arc<BTreeMap<String, Value>>,
        init_time: Time,
        finish_time: Time,
    ) -> RootSimulator {
        let rng = Rc::new(RefCell::new(StdRng::seed_from_u64(0)));
        let simulator = model_factory.build_simulator(
            &root_model_class_name,
            root_model_full_name.clone(),
            &global_resources,
        );

        RootSimulator {
            root_model_full_name,
            simulator,
            init_time,
            finish_time,
            sim_time: init_time,
            rng,
        }
    }

    pub fn init_static(
        &mut self,
        sim_dir: &PathBuf,
        init_variant: &BTreeMap<String, Value>,
        random_seed: u64,
    ) {
        self.rng = Rc::new(RefCell::new(StdRng::seed_from_u64(random_seed)));
        self.simulator.init_static(
            &self.root_model_full_name,
            sim_dir,
            init_variant,
            self.rng.clone(),
        );
    }

    pub fn init(&mut self) {
        self.simulator.init(self.init_time);
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

    fn finish(&mut self, sim_time: Time) -> Value {
        self.simulator.finish(sim_time);
        serde_json::Value::Bool(true)
    }

    fn step(&mut self) {
        self.collect_outputs();
        self.process_y_messages();
        self.process_x_messages();
    }

    pub fn run(&mut self) {
        while self.sim_time < self.finish_time {
            self.step();
            self.sim_time = self.simulator.t_next();
        }
        self.finish(self.sim_time);
    }
}
