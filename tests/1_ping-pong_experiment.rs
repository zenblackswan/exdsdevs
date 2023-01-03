use std::path::Path;

extern crate exdsdevs;
use exdsdevs::{
    dynamic::{DynamicFactory, DynamicFactoryStorage},
    experiment::{Experiment, InitVariantsFactory},
    logger::Logger,
    model::ModelFactory,
    observer::{ObserverFactory, ObserverFactoryStorage},
};

mod common;
use common::*;

#[test]
fn create_init_variants_factory() {
    let model_directory = Path::new("/home/zen/Work/soft_projects/exdsdevs/tests/models/ping_pong");
    let model_factory = ModelFactory::new(model_directory, Default::default(), Default::default());
    let init_variants_factory =
        InitVariantsFactory::new(model_factory.class_storage(), "ping-pong");
    dbg!(init_variants_factory);
}

#[test]
fn create_experiment_and_run_variants_without_log() {
    let mut dynamic_factory = DynamicFactoryStorage::new();
    dynamic_factory.add_dynamic_factory("root", DynamicFactory::<RootDynamic>::new());
    dynamic_factory.add_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());
    let experiment_path = Path::new(
        "/home/zen/Work/soft_projects/exdsdevs/tests/experiments/ping_pong/experiment.json",
    );
    let mut experiment = Experiment::new(experiment_path, dynamic_factory, Default::default());
    experiment.run_single_thread();
}

#[test]
fn run_experiment_single_thread_with_log() {
    let mut dynamic_factory = DynamicFactoryStorage::new();
    dynamic_factory.add_dynamic_factory("root", DynamicFactory::<RootDynamic>::new());
    dynamic_factory.add_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());

    let mut observer_factory = ObserverFactoryStorage::new();
    let logger_name = "std_logger";
    observer_factory.add_observer_factory(logger_name, ObserverFactory::<Logger>::new());

    let experiment_path = Path::new(
        "/home/zen/Work/soft_projects/exdsdevs/tests/experiments/ping_pong_log/experiment.json",
    );
    let mut experiment = Experiment::new(experiment_path, dynamic_factory, observer_factory);
    experiment.run_single_thread();
}

#[test]
fn run_experiment_multi_thread_with_log() {
    let mut dynamic_factory = DynamicFactoryStorage::new();
    dynamic_factory.add_dynamic_factory("root", DynamicFactory::<RootDynamic>::new());
    dynamic_factory.add_dynamic_factory("agent", DynamicFactory::<AgentDynamic>::new());

    let mut observer_factory = ObserverFactoryStorage::new();
    let logger_name = "std_logger";
    observer_factory.add_observer_factory(logger_name, ObserverFactory::<Logger>::new());

    let experiment_path = Path::new(
        "/home/zen/Work/soft_projects/exdsdevs/tests/experiments/ping_pong_log/experiment.json",
    );
    let mut experiment = Experiment::new(experiment_path, dynamic_factory, observer_factory);
    experiment.run_multi_thread();
}
