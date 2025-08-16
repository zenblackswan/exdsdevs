// Copyright 2023 Developers of the exdsdevs project.

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    collections::BTreeMap,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{spawn, JoinHandle},
};

use crate::{
    containers::{ModelSimResults, Value},
    model::Model,
    root_simulator::RootSimulator,
    time::Time,
};

pub trait ResultsAnalyzer {
    fn add_result(&mut self, thread_iter: u64, result: BTreeMap<String, ModelSimResults>);
    fn analyze(&mut self) -> Value;
}

pub struct Experiment<T: ResultsAnalyzer> {
    pub model: Model,
    pub init_time: Time,
    pub finish_time: Time,
    pub iterations: u64,
    pub random_seed: u64,
    pub results_analyzer: T,
}

struct ThreadData {
    model: Model,
    init_time: Time,
    finish_time: Time,
    random_seed: u64,
    iteration: u64,
}

struct ThreadResult {
    thread_number: u64,
    iteration: u64,
    result: BTreeMap<String, ModelSimResults>,
}

fn simulation(
    thread_number: u64,
    thread_data_rx: Receiver<Option<ThreadData>>,
    sim_results: Sender<ThreadResult>,
) {
    loop {
        let thread_data = thread_data_rx.recv().unwrap();
        if let Some(thread_data) = thread_data {
            let ThreadData {
                model,
                init_time,
                finish_time,
                random_seed,
                iteration,
            } = thread_data;
            let mut root_simulator = RootSimulator::new(model, iteration).unwrap();
            root_simulator.init(init_time, finish_time, random_seed);
            let result = root_simulator.run();
            let _ = sim_results.send(ThreadResult {
                thread_number,
                iteration,
                result,
            });
        } else {
            break;
        }
    }
}

impl<T: ResultsAnalyzer> Experiment<T> {
    pub fn check() {}

    fn generate_thread_data(&self, iteration: u64) -> ThreadData {
        ThreadData {
            model: self.model.clone(),
            init_time: self.init_time,
            finish_time: self.finish_time,
            random_seed: self.random_seed + iteration,
            iteration,
        }
    }

    pub fn run_single_thread(&mut self) -> Value {
        for iteration in 0..self.iterations {
            let ThreadData {
                model,
                init_time,
                finish_time,
                random_seed,
                iteration,
            } = self.generate_thread_data(iteration);
            let mut root_simulator = RootSimulator::new(model, iteration).unwrap();
            root_simulator.init(init_time, finish_time, random_seed);
            let result = root_simulator.run();
            self.results_analyzer.add_result(iteration, result);
        }
        self.results_analyzer.analyze()
    }

    pub fn run_multi_thread(&mut self, num_threads: u64) -> Value {
        let num_threads = if self.iterations < num_threads {
            self.iterations
        } else {
            num_threads
        };

        let (thread_data_txs, thread_data_rxs): (
            Vec<Sender<Option<ThreadData>>>,
            Vec<Receiver<Option<ThreadData>>>,
        ) = (0..num_threads).map(|_| channel()).collect();

        let (results_tx, results_rx) = channel();

        let thread_handles: Vec<JoinHandle<_>> = (0..num_threads)
            .zip(thread_data_rxs)
            .map(|(thread_number, thread_data_rx)| {
                spawn({
                    let sim_results = results_tx.clone();
                    move || simulation(thread_number, thread_data_rx, sim_results)
                })
            })
            .collect();

        let mut iteration = 0;

        for thread_number in 0..num_threads {
            let _ = thread_data_txs[thread_number as usize]
                .send(Some(self.generate_thread_data(iteration)));
            iteration += 1;
        }

        while iteration < self.iterations {
            let ThreadResult {
                thread_number,
                result,
                iteration: thread_iter,
            } = results_rx.recv().unwrap();
            self.results_analyzer.add_result(thread_iter, result);
            iteration += 1;
            let _ = thread_data_txs[thread_number as usize]
                .send(Some(self.generate_thread_data(iteration)));
        }

        for thread_data_tx in thread_data_txs {
            let _ = thread_data_tx.send(None);
        }

        for thread_handle in thread_handles {
            let _ = thread_handle.join();
        }

        self.results_analyzer.analyze()
    }
}
