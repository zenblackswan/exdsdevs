use std::env;

extern crate exdsdevs;
use exdsdevs::{
    dynamic::{DynamicFactory, DynamicFactoryStorage},
    experiment::Experiment,
    logger::Logger,
    observer::{ObserverFactory, ObserverFactoryStorage},
};
use ping_pong::{AgentDynamic, RootDynamic};

pub mod ping_pong;

pub fn main() {
    match env::args().nth(1) {
        Some(mode) if mode.as_str() == "single" => {
            println!("Running single thread");
            create_experiment().run_single_thread();
        }
        Some(mode) if mode.as_str() == "multi" => {
            println!("Running multi thread");
            create_experiment().run_multi_thread();
        }
        _ => {
            println!("ERROR: enter the parameter: `single` - for run in single thread, `multi` - for run in multi thread");
        }
    }
}

fn create_experiment() -> Experiment {
    let dynamic_factory = DynamicFactoryStorage::new()
        .with_dynamic_factory("root", DynamicFactory::<RootDynamic>::new())
        .with_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());
    let observer_factory = ObserverFactoryStorage::new()
        .with_observer_factory("std_logger", ObserverFactory::<Logger>::new());
    let experiment_path = env::current_dir()
        .map(|mut ep| {
            ep.push("examples/ping_pong/experiment_1.json");
            ep
        })
        .unwrap()
        .to_owned();
    Experiment::new(&experiment_path, dynamic_factory, observer_factory)
}
