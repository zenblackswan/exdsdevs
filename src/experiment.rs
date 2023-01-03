// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    collections::{BTreeMap, VecDeque},
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    containers::Value,
    dynamic::DynamicFactoryStorage,
    model::{ModelClass, ModelFactory},
    observer::ObserverFactoryStorage,
    root_simulator::RootSimulator,
    time::Time,
};

#[derive(Debug, Serialize, Deserialize)]
struct ExperimentConfig {
    name: String,
    experiment_directory: Option<PathBuf>,
    results_directory: String,
    model_directory: String,
    root_model_class: String,
    init_time: String,
    finish_time: String,
    random_seed: u64,
    iterations: u64,
    global_resources: BTreeMap<String, String>,
}

impl ExperimentConfig {
    fn new(experiment_path: &Path) -> ExperimentConfig {
        let experiment_json_string = read_to_string(experiment_path).unwrap();
        let mut experiment_config =
            serde_json::from_str::<ExperimentConfig>(&experiment_json_string).unwrap();
        let experiment_directory = experiment_path.parent().unwrap().to_path_buf();
        experiment_config.experiment_directory = Some(experiment_directory);
        experiment_config
    }

    fn experiment_name(&self) -> String {
        self.name.clone()
    }

    fn experiment_directory(&self) -> PathBuf {
        self.experiment_directory.as_ref().unwrap().clone()
    }

    fn root_model_class(&self) -> String {
        self.root_model_class.clone()
    }

    fn init_time(&self) -> Time {
        Time::Value(
            i128::from_str(&self.init_time)
                .unwrap_or_else(|_| panic!("Cannot convert value {} to Time", &self.init_time)),
        )
    }

    fn finish_time(&self) -> Time {
        match self.finish_time.as_str() {
            "Infinity" => Time::Inf,
            t => {
                let time = Time::Value(
                    i128::from_str(t)
                        .unwrap_or_else(|_| panic!("Cannot convert value {} to Time", t)),
                );
                if time < self.init_time() {
                    panic!("finish_time cannot be lesser then init_time");
                } else {
                    time
                }
            }
        }
    }

    fn model_directory(&self) -> PathBuf {
        let config_model_directory = PathBuf::from(&self.model_directory);
        if config_model_directory.is_absolute() {
            config_model_directory
        } else {
            let mut model_directory = PathBuf::from(self.experiment_directory.as_ref().unwrap());
            model_directory.push(config_model_directory);
            model_directory
        }
    }

    fn results_directory(&self) -> PathBuf {
        let config_results_directory = PathBuf::from(&self.results_directory);
        if config_results_directory.is_absolute() {
            config_results_directory
        } else {
            let mut results_directory = PathBuf::from(self.experiment_directory.as_ref().unwrap());
            results_directory.push(config_results_directory);
            results_directory
        }
    }

    fn random_seed(&self) -> u64 {
        self.random_seed
    }

    fn iterations(&self) -> u64 {
        self.iterations
    }

    fn global_resources(&self) -> &BTreeMap<String, String> {
        &self.global_resources
    }
}

pub struct Experiment {
    pub experiment_name: String,
    pub experiment_directory: PathBuf,
    pub model_directory: PathBuf,
    pub results_directory: PathBuf,
    pub root_model_full_name: String,
    pub root_model_class_name: String,
    pub model_factory: Arc<ModelFactory>,
    pub global_resources: Arc<BTreeMap<String, Value>>,
    pub init_time: Time,
    pub finish_time: Time,
    pub random_seed: u64,
    pub iterations: u64,
    pub init_variants_factory: InitVariantsFactory,
}

impl Experiment {
    pub fn new(
        experiment_path: &Path,
        dynamic_factory: DynamicFactoryStorage,
        observer_factory: ObserverFactoryStorage,
    ) -> Self {
        let experiment_config = ExperimentConfig::new(experiment_path);

        let experiment_name = experiment_config.experiment_name();
        let experiment_directory = experiment_config.experiment_directory();
        let model_directory = experiment_config.model_directory();
        let results_directory = experiment_config.results_directory();
        let root_model_full_name: String = "root".to_owned();
        let root_model_class_name = experiment_config.root_model_class();
        let model_factory = Arc::new(ModelFactory::new(
            &model_directory,
            dynamic_factory,
            observer_factory,
        ));
        let init_variants_factory =
            InitVariantsFactory::new(model_factory.class_storage(), &root_model_class_name);
        let global_resources = Arc::new(Self::build_global_resources(&experiment_config));
        let init_time = experiment_config.init_time();
        let finish_time = experiment_config.finish_time();
        let random_seed = experiment_config.random_seed();
        let iterations = experiment_config.iterations();

        Self {
            experiment_name,
            experiment_directory,
            model_directory,
            results_directory,
            root_model_full_name,
            root_model_class_name,
            model_factory,
            global_resources,
            init_time,
            finish_time,
            random_seed,
            iterations,
            init_variants_factory,
        }
    }

    pub fn run_single_thread(&mut self) {
        while let Some((var_number, init_variant)) =
            self.init_variants_factory.next_enumerated_variant()
        {
            let init_variant = Arc::new(init_variant);
            for iteration in 0..self.iterations {
                let random_seed = self.random_seed + iteration;
                let results_directory = self.results_directory.clone();
                let model_factory = self.model_factory.clone();
                let root_model_class_name = self.root_model_class_name.clone();
                let root_model_full_name = self.root_model_full_name.clone();
                let global_resources = self.global_resources.clone();
                let init_time = self.init_time;
                let finish_time = self.finish_time;
                let init_variant = init_variant.clone();

                let mut root = Self::create_root_simulator(
                    results_directory,
                    model_factory,
                    root_model_class_name,
                    root_model_full_name,
                    global_resources,
                    init_time,
                    finish_time,
                    var_number,
                    iteration,
                    random_seed,
                    init_variant,
                );
                root.init();
                root.run();
            }
        }
    }

    pub fn run_multi_thread(&mut self) {
        while let Some((var_number, init_variant)) =
            self.init_variants_factory.next_enumerated_variant()
        {
            let pool = threadpool::Builder::new().build();
            let init_variant = Arc::new(init_variant);

            for iteration in 0..self.iterations {
                let random_seed = self.random_seed + iteration;
                let results_directory = self.results_directory.clone();
                let model_factory = self.model_factory.clone();
                let root_model_class_name = self.root_model_class_name.clone();
                let root_model_full_name = self.root_model_full_name.clone();
                let global_resources = self.global_resources.clone();
                let init_time = self.init_time;
                let finish_time = self.finish_time;
                let init_variant = init_variant.clone();
                pool.execute(move || {
                    let mut root = Self::create_root_simulator(
                        results_directory,
                        model_factory,
                        root_model_class_name,
                        root_model_full_name,
                        global_resources,
                        init_time,
                        finish_time,
                        var_number,
                        iteration,
                        random_seed,
                        init_variant,
                    );
                    root.init();
                    root.run();
                });
            }
            pool.join();
        }
    }

    fn build_global_resources(experiment_config: &ExperimentConfig) -> BTreeMap<String, Value> {
        let experiment_directory = experiment_config.experiment_directory();
        experiment_config
            .global_resources()
            .iter()
            .map(|(resource_name, resource_value_str_path)| {
                let mut rv_path = PathBuf::from_str(resource_value_str_path).unwrap_or_else(|_| {
                    panic!("Resource {} has wrong name", &resource_value_str_path)
                });
                if !rv_path.is_absolute() {
                    rv_path = {
                        let mut tmp_path = experiment_directory.to_path_buf();
                        tmp_path.push(rv_path);
                        tmp_path
                    };
                }
                let resource_value_string = read_to_string(rv_path).unwrap();
                let resource_value = serde_json::from_str::<Value>(&resource_value_string).unwrap();
                (resource_name.clone(), resource_value)
            })
            .collect::<BTreeMap<String, Value>>()
    }

    #[allow(clippy::too_many_arguments)]
    fn create_root_simulator(
        results_directory: PathBuf,
        model_factory: Arc<ModelFactory>,
        root_model_class_name: String,
        root_model_full_name: String,
        global_resources: Arc<BTreeMap<String, Value>>,
        init_time: Time,
        finish_time: Time,
        var_number: u64,
        iteration: u64,
        random_seed: u64,
        init_variant: Arc<BTreeMap<String, Value>>,
    ) -> RootSimulator {
        let mut sim_dir = results_directory;
        sim_dir.push(PathBuf::from(format!(
            "var_{}/iter_{}",
            var_number, iteration
        )));
        let mut root_simulator = RootSimulator::new(
            model_factory,
            root_model_class_name,
            root_model_full_name,
            global_resources,
            init_time,
            finish_time,
        );
        root_simulator.init_static(&sim_dir, &init_variant, random_seed);
        root_simulator
    }
}

#[derive(Debug)]
pub struct InitVariantsFactory {
    init_variants_values: BTreeMap<String, BTreeMap<String, Value>>,
    init_variants_names: BTreeMap<String, Vec<String>>,
    init_vec: Vec<VarDigit>,
    carry: usize,
    var_number: u64,
}

#[derive(Debug)]
struct VarDigit {
    model_path: String,
    len: usize,
    idx: usize,
}

impl InitVariantsFactory {
    pub fn new(class_storage: &BTreeMap<String, ModelClass>, root_model_class_name: &str) -> Self {
        let mut init_variants_values: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
        let mut default_init_values: BTreeMap<String, Value> = BTreeMap::new();
        let mut model_stack: VecDeque<(String, &ModelClass)> = VecDeque::new();
        let root_model_full_name: String = "root".to_string();
        let root_model_class = class_storage.get(root_model_class_name).unwrap();
        let root_default_init_values = root_model_class.get_default_init();
        let root_init_variants = root_model_class.root_init_variants().unwrap_or_else(|| {
            panic!(
                "Error: no value set for root_init for {}",
                root_model_class_name
            )
        });
        init_variants_values.insert(root_model_full_name.clone(), root_init_variants);
        default_init_values.insert(root_model_full_name.clone(), root_default_init_values);
        model_stack.push_back((root_model_full_name, root_model_class));

        while let Some((model_full_name, model_class)) = model_stack.pop_front() {
            for (submodel_name, submodel) in model_class.submodels_iter() {
                let submodel_full_name = format!("{}/{}", &model_full_name, submodel_name);
                let submodel_init_variants = submodel.get_init_values();
                init_variants_values.insert(submodel_full_name.clone(), submodel_init_variants);
                let submodel_class = class_storage.get(submodel.model_class()).unwrap();
                let default_init_variant = submodel_class.get_default_init();
                default_init_values.insert(submodel_full_name.clone(), default_init_variant);
                model_stack.push_back((submodel_full_name, submodel_class));
            }
        }

        let init_variants_names = init_variants_values
            .iter()
            .map(|(model_full_name, model_init_values)| {
                (
                    model_full_name.clone(),
                    model_init_values.keys().cloned().collect(),
                )
            })
            .collect();

        let init_vec = Self::get_init_vec(&init_variants_names);
        let carry: usize = 0;
        let var_number: u64 = 0;
        Self {
            init_variants_values,
            init_variants_names,
            init_vec,
            carry,
            var_number,
        }
    }

    fn get_init_vec(model_init_variants: &BTreeMap<String, Vec<String>>) -> Vec<VarDigit> {
        model_init_variants
            .iter()
            .map(|(model_path, init_varians)| VarDigit {
                model_path: model_path.clone(),
                len: init_varians.len(),
                idx: 0,
            })
            .collect()
    }

    pub fn next_enumerated_variant(&mut self) -> Option<(u64, BTreeMap<String, Value>)> {
        self.next_variant().map(|(var_number, next_variant)| {
            let mut next_variant_values: BTreeMap<String, Value> = BTreeMap::new();
            for (model_full_name, variant_name) in &next_variant {
                if let Some(variants) = self.init_variants_values.get(model_full_name) {
                    if let Some(variant_value) = variants.get(variant_name) {
                        next_variant_values
                            .insert(model_full_name.to_owned(), variant_value.clone());
                    }
                }
            }
            (var_number, next_variant_values)
        })
    }

    pub fn next_variant(&mut self) -> Option<(u64, BTreeMap<String, String>)> {
        if self.carry == 0 {
            let next_init = self
                .init_vec
                .iter()
                .map(|var_digit| {
                    (
                        var_digit.model_path.clone(),
                        self.init_variants_names
                            .get(&var_digit.model_path)
                            .unwrap()
                            .get(var_digit.idx)
                            .unwrap()
                            .clone(),
                    )
                })
                .collect();

            self.carry = 1;
            for val in self.init_vec.iter_mut().rev() {
                val.idx += self.carry;
                if val.idx == val.len {
                    self.carry = 1;
                    val.idx = 0;
                } else {
                    self.carry = 0;
                };
            }
            let var_number = self.var_number;
            self.var_number += 1;
            Some((var_number, next_init))
        } else {
            None
        }
    }
}
