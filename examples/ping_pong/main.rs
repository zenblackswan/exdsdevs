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
    if let Some(mode) = env::args().nth(1) {
        if mode.as_str() == "single" {
            println!("Running in single thread");
            let mut experiment = create_experiment();
            experiment.run_single_thread();
        } else if mode.as_str() == "multi" {
            println!("Running in milti thread");
            let mut experiment = create_experiment();
            experiment.run_multi_thread();
        } else {
            println!("ERROR: enter the parameter: single - for run in single thread, multi - for run in multi thread");
        }
    } else {
        println!("ERROR: enter the parameter: single - for run in single thread, multi - for run in multi thread");
    }
}

fn create_experiment() -> Experiment {
    let mut dynamic_factory = DynamicFactoryStorage::new();
    dynamic_factory.add_dynamic_factory("root", DynamicFactory::<RootDynamic>::new());
    dynamic_factory.add_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());
    let mut observer_factory = ObserverFactoryStorage::new();
    let logger_name = "std_logger";
    observer_factory.add_observer_factory(logger_name, ObserverFactory::<Logger>::new());
    let mut experiment_path = env::current_dir().unwrap().to_owned();
    experiment_path.push("examples/ping_pong/experiment_1.json");
    Experiment::new(&experiment_path, dynamic_factory, observer_factory)
}
