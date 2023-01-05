use std::{
    env,
    path::{Path, PathBuf},
};

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
    let mode = match env::args().nth(1) {
        Some(mode) if mode.as_str() == "single" => mode,
        Some(mode) if mode.as_str() == "multi" => mode,
        _ => {
            panic!("ERROR: enter the parameter: `single` - for run in single thread, `multi` - for run in multi thread");
        }
    };

    let experiment_path = match env::args().nth(2) {
        Some(path_string) => {
            let path = PathBuf::from(&path_string);
            let exp_path = if !path.is_absolute() {
                let mut path = env::current_dir().unwrap();
                path.push(&path_string);
                path
            } else {
                path
            };
            if exp_path.exists() {
                exp_path
            } else {
                panic!("Path {} does not exist", &exp_path.to_string_lossy());
            }
        }
        None => panic!("ERROR: enter the path to experiment config"),
    };
    let mut experiment = create_experiment(&experiment_path);
    match mode.as_str() {
        "single" => {
            println!("Running single thread");
            experiment.run_single_thread();
        }
        "multi" => {
            println!("Running multi thread");
            experiment.run_multi_thread();
        }
        _ => {}
    }
}

fn create_experiment(experiment_path: &Path) -> Experiment {
    let dynamic_factory = DynamicFactoryStorage::new()
        .with_dynamic_factory("root", DynamicFactory::<RootDynamic>::new())
        .with_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());
    let observer_factory = ObserverFactoryStorage::new()
        .with_observer_factory("std_logger", ObserverFactory::<Logger>::new());
    Experiment::new(&experiment_path, dynamic_factory, observer_factory)
}
