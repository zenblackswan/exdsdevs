// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::containers::{Bag, Mail, MailItem, Msg, Value};
use crate::dynamic::{Dynamic, DynamicFactoryStorage};
use crate::observer::ObserverFactoryStorage;
use crate::simulator::Simulator;
use crate::time::Time;

use std::collections::btree_map::{Iter, IterMut};
use std::collections::VecDeque;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

#[derive(Default, Debug, Clone)]
pub struct Resources {
    pub local: Value,
    pub global: Arc<BTreeMap<String, Value>>,
}

#[derive(Clone)]
pub struct ExternalInputCoupling {
    pub source_port: String,
    pub destination_model: String,
    pub destination_model_port: String,
}

impl ExternalInputCoupling {
    pub fn new(source_port: &str, destination_model: &str, destination_model_port: &str) -> Self {
        ExternalInputCoupling {
            source_port: source_port.to_string(),
            destination_model: destination_model.to_string(),
            destination_model_port: destination_model_port.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct InternalCoupling {
    pub source_model: String,
    pub source_model_port: String,
    pub destination_model: String,
    pub destination_model_port: String,
}

impl InternalCoupling {
    pub fn new(
        source_model: &str,
        source_model_port: &str,
        destination_model: &str,
        destination_model_port: &str,
    ) -> InternalCoupling {
        InternalCoupling {
            source_model: source_model.to_string(),
            source_model_port: source_model_port.to_string(),
            destination_model: destination_model.to_string(),
            destination_model_port: destination_model_port.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ExternalOutputCoupling {
    pub source_model: String,
    pub source_model_port: String,
    pub destination_port: String,
}

impl ExternalOutputCoupling {
    pub fn new(source_model: &str, source_model_port: &str, destination_port: &str) -> Self {
        ExternalOutputCoupling {
            source_model: source_model.to_string(),
            source_model_port: source_model_port.to_string(),
            destination_port: destination_port.to_string(),
        }
    }
}

pub struct Model {
    pub structure: Structure,
    pub dynamic: Box<dyn Dynamic>,
}

impl Model {
    pub fn new(structure: Structure, dynamic: Box<dyn Dynamic>) -> Self {
        Model { structure, dynamic }
    }

    pub(crate) fn has_submodels(&self) -> bool {
        !self.structure.sub_simulators.is_empty()
    }

    pub(crate) fn sub_simulators(&mut self) -> IterMut<String, Simulator> {
        self.structure.sub_simulators.iter_mut()
    }

    pub(crate) fn external_input_couplings(&self) -> &[ExternalInputCoupling] {
        &self.structure.external_input_couplings
    }

    pub(crate) fn internal_couplings(&self) -> &[InternalCoupling] {
        &self.structure.internal_couplings
    }

    pub(crate) fn external_output_couplings(&self) -> &[ExternalOutputCoupling] {
        &self.structure.external_output_couplings
    }

    pub(crate) fn get_subsimulators(&mut self, simulator_name: &str) -> &mut Simulator {
        self.structure
            .sub_simulators
            .get_mut(simulator_name)
            .unwrap()
    }

    pub(crate) fn get_y_bag_from_mail(&self, mail: &Mail) -> Bag {
        let mut bag: Bag = Bag::new();
        for ExternalOutputCoupling {
            source_model,
            source_model_port,
            destination_port,
        } in self.external_output_couplings()
        {
            for MailItem { model_name, y_bag } in mail.iter() {
                if model_name == source_model {
                    for Msg { port, value } in y_bag {
                        if source_model_port == port {
                            bag.push(Msg {
                                port: destination_port.clone(),
                                value: value.clone(),
                            });
                        }
                    }
                }
            }
        }
        bag
    }

    pub(crate) fn init(
        &mut self,
        init_time: Time,
        init_value: &Value,
        resources: &Resources,
        rng: &mut StdRng,
    ) {
        self.dynamic
            .init(&mut self.structure, init_time, init_value, resources, rng)
    }

    pub(crate) fn time_advance(&self, rng: &mut StdRng) -> Time {
        self.dynamic.time_advance(&self.structure, rng)
    }

    pub(crate) fn state(&self) -> Value {
        self.dynamic.state()
    }

    pub(crate) fn output(&self, sim_time: Time) -> Bag {
        self.dynamic.output(&self.structure, sim_time)
    }

    pub(crate) fn internal_transition(&mut self, sim_time: Time, rng: &mut StdRng) {
        self.dynamic
            .internal_transition(&mut self.structure, sim_time, rng);
    }

    pub(crate) fn external_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
        self.dynamic
            .external_transition(&self.structure, sim_time, elapsed, x_bag, rng)
    }

    pub(crate) fn external_mail_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        mail: &Mail,
        rng: &mut StdRng,
    ) {
        self.dynamic
            .external_mail_transition(&mut self.structure, sim_time, elapsed, mail, rng)
    }

    pub(crate) fn confluent_transition(&mut self, sim_time: Time, x_bag: &Bag, rng: &mut StdRng) {
        self.dynamic
            .confluent_transition(&mut self.structure, sim_time, x_bag, rng);
    }

    pub(crate) fn finish(&mut self, sim_time: Time) {
        self.dynamic.finish(sim_time);
    }
}

pub struct Structure {
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub sub_simulators: BTreeMap<String, Simulator>,
    pub external_input_couplings: Vec<ExternalInputCoupling>,
    pub internal_couplings: Vec<InternalCoupling>,
    pub external_output_couplings: Vec<ExternalOutputCoupling>,
}

impl Structure {
    pub fn new(
        input_ports: &[&str],
        output_ports: &[&str],
        submodels: BTreeMap<String, Simulator>,
        external_input_couplings: &[(&str, &str, &str)],
        internal_couplings: &[(&str, &str, &str, &str)],
        external_output_couplings: &[(&str, &str, &str)],
    ) -> Self {
        Self {
            input_ports: input_ports.iter().map(|&port| port.to_string()).collect(),
            output_ports: output_ports.iter().map(|&port| port.to_string()).collect(),
            sub_simulators: submodels,
            external_input_couplings: external_input_couplings
                .iter()
                .map(|&(src_port, dst_model, dst_model_port)| {
                    ExternalInputCoupling::new(src_port, dst_model, dst_model_port)
                })
                .collect(),
            internal_couplings: internal_couplings
                .iter()
                .map(|&(src_model, src_port, dst_model, dst_model_port)| {
                    InternalCoupling::new(src_model, src_port, dst_model, dst_model_port)
                })
                .collect(),
            external_output_couplings: external_output_couplings
                .iter()
                .map(|&(src_model, src_port, dst_port)| {
                    ExternalOutputCoupling::new(src_model, src_port, dst_port)
                })
                .collect(),
        }
    }
}

#[derive(Default)]
pub struct ModelFactory {
    class_storage: BTreeMap<String, ModelClass>,
    dynamic_factory_storage: DynamicFactoryStorage,
    observer_factory_storage: ObserverFactoryStorage,
}

impl ModelFactory {
    pub fn new(
        model_directory: &Path,
        dynamic_factory_storage: DynamicFactoryStorage,
        observer_factory_storage: ObserverFactoryStorage,
    ) -> Self {
        let class_files = Self::prepare_paths(model_directory);
        let class_storage = Self::parse_class_files(class_files.as_slice());
        ModelFactory {
            class_storage,
            dynamic_factory_storage,
            observer_factory_storage,
        }
    }

    pub fn class_storage(&self) -> &BTreeMap<String, ModelClass> {
        &self.class_storage
    }

    pub fn set_dynamic_factory(&mut self, dynamic_factory: DynamicFactoryStorage) {
        self.dynamic_factory_storage = dynamic_factory;
    }

    pub fn set_observer_factory(&mut self, observer_factory: ObserverFactoryStorage) {
        self.observer_factory_storage = observer_factory;
    }

    fn prepare_paths(model_directory: &Path) -> Vec<PathBuf> {
        let mut class_paths: Vec<PathBuf> = Vec::new();
        let mut dirs: VecDeque<PathBuf> = VecDeque::new();
        dirs.push_back(model_directory.to_path_buf());
        while let Some(dir) = dirs.pop_front() {
            if let Ok(mut dir_entries) = dir.read_dir() {
                for entry in dir_entries.by_ref().flatten() {
                    match entry.file_type() {
                        Ok(item) => {
                            if item.is_file() && !item.is_symlink() {
                                match entry.path().extension() {
                                    Some(ext) if ext == "json" => class_paths.push(entry.path()),
                                    _ => {}
                                }
                            } else if item.is_dir() && !item.is_symlink() {
                                dirs.push_back(entry.path())
                            }
                        }
                        Err(_) => panic!("Error while reding model files"),
                    }
                }
            }
        }
        class_paths
    }

    fn parse_class_files(class_files: &[PathBuf]) -> BTreeMap<String, ModelClass> {
        class_files
            .iter()
            .map(|local_path| read_to_string(local_path).unwrap())
            .map(|json_string| serde_json::from_str::<ModelClass>(&json_string).unwrap())
            .map(|model_class| (model_class.model_class(), model_class))
            .collect::<BTreeMap<String, ModelClass>>()
    }

    pub(crate) fn build_simulator(
        &self,
        model_class_name: &str,
        model_full_name: String,
        global_resources: &Arc<BTreeMap<String, Value>>,
    ) -> Simulator {
        let model_class = self
            .class_storage
            .get(model_class_name)
            .unwrap_or_else(|| panic!("Model class '{}' was not registered", model_class_name));
        let dynamic = self
            .dynamic_factory_storage
            .get_dynamic(model_class.dynamic_type())
            .unwrap_or_else(|_| {
                panic!(
                    "Model dynamic '{}' was not registered",
                    model_class.dynamic_type()
                )
            });

        let sub_simulators = model_class
            .submodels_iter()
            .map(|(submodel_name, submodel)| {
                let submodel_full_name = format!("{}/{}", model_full_name, submodel_name);
                let model = self.build_simulator(
                    submodel.model_class(),
                    submodel_full_name,
                    global_resources,
                );
                (submodel_name.clone(), model)
            })
            .collect::<BTreeMap<String, Simulator>>();

        let external_input_couplings: Vec<ExternalInputCoupling> = model_class
            .external_input_couplings()
            .iter()
            .map(From::from)
            .collect();
        let internal_couplings: Vec<InternalCoupling> = model_class
            .internal_couplings()
            .iter()
            .map(From::from)
            .collect();
        let external_output_couplings: Vec<ExternalOutputCoupling> = model_class
            .external_output_couplings()
            .iter()
            .map(From::from)
            .collect();

        let structure = Structure {
            input_ports: model_class.input_ports().to_vec(),
            output_ports: model_class.output_ports().to_vec(),
            sub_simulators,
            external_input_couplings,
            internal_couplings,
            external_output_couplings,
        };

        let model = Model::new(structure, dynamic);

        let resources = Resources {
            local: model_class.local_resources(),
            global: global_resources.clone(),
        };

        let mut simulator = Simulator::new(&model_full_name, model, resources);
        for ObserverClass {
            observer_class,
            observer_config,
        } in model_class.observers()
        {
            let mut observer = self
                .observer_factory_storage
                .get_observer(observer_class)
                .unwrap_or_else(|_| {
                    panic!(
                        "Observer with name '{}' has not been registered",
                        observer_class
                    )
                });
            observer.config(observer_config);
            simulator.add_observer(observer);
        }
        simulator
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Submodel {
    model_class: String,
    init_variants: Option<BTreeMap<String, Value>>,
}

impl Submodel {
    pub(crate) fn model_class(&self) -> &str {
        self.model_class.as_str()
    }

    pub(crate) fn get_init_values(&self) -> BTreeMap<String, Value> {
        match &self.init_variants {
            Some(init_variants) => init_variants.clone(),
            None => BTreeMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ModelClassExtInCoupl {
    src_port: String,
    dst_model: String,
    dst_port: String,
}

impl From<&ModelClassExtInCoupl> for ExternalInputCoupling {
    fn from(ext_in_coupl: &ModelClassExtInCoupl) -> Self {
        let ModelClassExtInCoupl {
            src_port,
            dst_model,
            dst_port,
        } = ext_in_coupl;
        ExternalInputCoupling {
            source_port: src_port.clone(),
            destination_model: dst_model.clone(),
            destination_model_port: dst_port.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ModelClassIntCoupl {
    src_model: String,
    src_port: String,
    dst_model: String,
    dst_port: String,
}

impl From<&ModelClassIntCoupl> for InternalCoupling {
    fn from(int_coupl: &ModelClassIntCoupl) -> Self {
        let ModelClassIntCoupl {
            src_model,
            src_port,
            dst_model,
            dst_port,
        } = int_coupl;
        InternalCoupling {
            source_model: src_model.clone(),
            source_model_port: src_port.clone(),
            destination_model: dst_model.clone(),
            destination_model_port: dst_port.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ModelClassExtOutCoupl {
    src_model: String,
    src_port: String,
    dst_port: String,
}

impl From<&ModelClassExtOutCoupl> for ExternalOutputCoupling {
    fn from(ext_out_coupl: &ModelClassExtOutCoupl) -> Self {
        let ModelClassExtOutCoupl {
            src_model,
            src_port,
            dst_port,
        } = ext_out_coupl;
        ExternalOutputCoupling {
            source_model: src_model.clone(),
            source_model_port: src_port.clone(),
            destination_port: dst_port.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelClass {
    model_class: String,
    input_ports: Vec<String>,
    output_ports: Vec<String>,
    dynamic_type: String,
    submodels: BTreeMap<String, Submodel>,
    external_input_couplings: Vec<ModelClassExtInCoupl>,
    internal_couplings: Vec<ModelClassIntCoupl>,
    external_output_couplings: Vec<ModelClassExtOutCoupl>,
    default_init: Value,
    root_init_variants: Option<BTreeMap<String, Value>>,
    local_resources: Value,
    observers: Vec<ObserverClass>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObserverClass {
    observer_class: String,
    observer_config: Value,
}

impl ModelClass {
    fn model_class(&self) -> String {
        self.model_class.clone()
    }

    pub(crate) fn get_default_init(&self) -> Value {
        self.default_init.clone()
    }

    fn dynamic_type(&self) -> &str {
        self.dynamic_type.as_str()
    }

    pub(crate) fn submodels_iter(&self) -> Iter<std::string::String, Submodel> {
        self.submodels.iter()
    }

    fn input_ports(&self) -> &[String] {
        self.input_ports.as_slice()
    }

    fn output_ports(&self) -> &[String] {
        self.output_ports.as_slice()
    }

    fn external_input_couplings(&self) -> &[ModelClassExtInCoupl] {
        self.external_input_couplings.as_slice()
    }

    fn internal_couplings(&self) -> &[ModelClassIntCoupl] {
        self.internal_couplings.as_slice()
    }

    fn external_output_couplings(&self) -> &[ModelClassExtOutCoupl] {
        self.external_output_couplings.as_slice()
    }

    fn local_resources(&self) -> Value {
        self.local_resources.clone()
    }

    pub(crate) fn root_init_variants(&self) -> Option<BTreeMap<String, Value>> {
        self.root_init_variants.as_ref().cloned()
    }

    fn observers(&self) -> &[ObserverClass] {
        self.observers.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_paths() {
        let pth = Path::new("/home/zen/Work/soft_projects/exdsdevs/tests/models/ping_pong");
        let col_pths = ModelFactory::prepare_paths(pth);
        println!("Файлы: {:#?}", col_pths);
    }

    #[test]
    fn test_read_class_paths() {
        let pth = Path::new("/home/zen/Work/soft_projects/exdsdevs/tests/models/ping_pong");
        let col_pths = ModelFactory::prepare_paths(pth);
        println!("Файлы: {:#?}", col_pths);
        let _classes = ModelFactory::parse_class_files(&col_pths);
    }
}
