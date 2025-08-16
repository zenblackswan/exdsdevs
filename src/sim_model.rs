// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use rand::rngs::StdRng;

use crate::containers::{Bag, Mail, MailItem, ModelSimResults, Msg, Outputs, Value};
use crate::dynamic::Dynamic;
use crate::model::Model;
use crate::simulator::Simulator;
use crate::time::Time;

use std::cell::RefCell;
pub use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct SimModel {
    pub full_name: String,
    pub dynamic: Box<dyn Dynamic>,
    pub structure: Option<Structure>,
    // TODO: сделать разделяемой ссылкой, доступной только для чтения
    // pub resources: Rc<BTreeMap<String, Value>>,
}

impl SimModel {
    pub fn new(full_name: String, model: Model, rng: &Rc<RefCell<StdRng>>, iteration: u64) -> Self {
        let Model {
            dynamic,
            sumbodels,
            input_couplings,
            internal_couplings,
            output_couplings,
            ..
        } = model;
        let mut external_input_couplings: BTreeMap<String, BTreeMap<String, Vec<String>>> =
            Default::default();
        for (self_in_port, submodel, submodel_in_port) in input_couplings {
            if !external_input_couplings.contains_key(&self_in_port) {
                external_input_couplings.insert(self_in_port.clone(), Default::default());
            }
            let submodels_and_ports = external_input_couplings.get_mut(&self_in_port).unwrap();
            if !submodels_and_ports.contains_key(&submodel) {
                submodels_and_ports.insert(submodel.clone(), Default::default());
            }

            let submodels_ports = submodels_and_ports.get_mut(&submodel).unwrap();
            if !submodels_ports.contains(&submodel_in_port) {
                submodels_ports.push(submodel_in_port);
            }
        }

        let mut self_internal_couplings: BTreeMap<
            String,
            BTreeMap<String, BTreeMap<String, Vec<String>>>,
        > = Default::default();
        for (
            source_submodel,
            source_submodel_out_port,
            destination_model,
            destination_model_out_port,
        ) in internal_couplings
        {
            if !self_internal_couplings.contains_key(&source_submodel) {
                self_internal_couplings.insert(source_submodel.clone(), Default::default());
            }
            let submodels_and_its_couplings =
                self_internal_couplings.get_mut(&source_submodel).unwrap();
            if !submodels_and_its_couplings.contains_key(&source_submodel_out_port) {
                submodels_and_its_couplings
                    .insert(source_submodel_out_port.clone(), Default::default());
            }

            let submodels_and_ports = submodels_and_its_couplings
                .get_mut(&source_submodel_out_port)
                .unwrap();
            if !submodels_and_ports.contains_key(&destination_model) {
                submodels_and_ports.insert(destination_model.clone(), Default::default());
            }

            let destination_submodels_ports =
                submodels_and_ports.get_mut(&destination_model).unwrap();
            if !destination_submodels_ports.contains(&destination_model_out_port) {
                destination_submodels_ports.push(destination_model_out_port);
            }
        }

        let mut external_output_couplings: BTreeMap<String, BTreeMap<String, Vec<String>>> =
            Default::default();
        for (submodel, submodel_out_port, self_out_port) in output_couplings {
            if !external_output_couplings.contains_key(&submodel) {
                external_output_couplings.insert(submodel.clone(), Default::default());
            }
            let submodel_out_and_self_out_ports =
                external_output_couplings.get_mut(&submodel).unwrap();
            if !submodel_out_and_self_out_ports.contains_key(&submodel_out_port) {
                submodel_out_and_self_out_ports
                    .insert(submodel_out_port.clone(), Default::default());
            }

            let self_out_ports = submodel_out_and_self_out_ports
                .get_mut(&submodel_out_port)
                .unwrap();
            if !self_out_ports.contains(&self_out_port) {
                self_out_ports.push(self_out_port);
            }
        }

        let sub_simulators: BTreeMap<String, Simulator> = sumbodels
            .into_iter()
            .map({
                |(sub_sim_model_name, sub_model)| {
                    let sub_sim_model_full_name = format!("{}/{}", &full_name, sub_sim_model_name);
                    let sub_simulator = Simulator::new(
                        sub_sim_model_full_name.clone(),
                        sub_model,
                        rng.clone(),
                        iteration,
                    );
                    (sub_sim_model_name, sub_simulator)
                }
            })
            .collect();

        let structure = if !sub_simulators.is_empty() {
            Some(Structure {
                sub_simulators,
                external_input_couplings,
                internal_couplings: self_internal_couplings,
                external_output_couplings,
            })
        } else {
            None
        };

        Self {
            full_name,
            dynamic,
            structure,
        }
    }

    pub(crate) fn full_name(&self) -> &str {
        &self.full_name
    }

    pub(crate) fn has_submodels(&self) -> bool {
        self.structure.is_some()
    }

    pub(crate) fn sub_simulators(&mut self) -> Option<&mut BTreeMap<String, Simulator>> {
        self.structure
            .as_mut()
            .and_then(|Structure { sub_simulators, .. }| Some(sub_simulators))
    }

    pub(crate) fn external_input_couplings(
        &self,
    ) -> Option<&BTreeMap<String, BTreeMap<String, Vec<String>>>> {
        self.structure.as_ref().and_then(
            |Structure {
                 external_input_couplings,
                 ..
             }| Some(external_input_couplings),
        )
    }

    pub(crate) fn internal_couplings(
        &self,
    ) -> Option<&BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<String>>>>> {
        self.structure.as_ref().and_then(
            |Structure {
                 internal_couplings, ..
             }| Some(internal_couplings),
        )
    }

    pub(crate) fn external_output_couplings(
        &self,
    ) -> Option<&BTreeMap<String, BTreeMap<String, Vec<String>>>> {
        self.structure.as_ref().and_then(
            |Structure {
                 external_output_couplings,
                 ..
             }| Some(external_output_couplings),
        )
    }

    pub(crate) fn get_subsimulator(&mut self, simulator_name: &str) -> Option<&mut Simulator> {
        self.structure
            .as_mut()
            .and_then(|Structure { sub_simulators, .. }| sub_simulators.get_mut(simulator_name))
    }

    pub(crate) fn get_y_bag_from_mail(&self, mail: &Mail) -> Bag {
        let eoc = self.external_output_couplings().unwrap();
        let mut bag: Bag = Bag::new();
        for MailItem { model_name, y_bag } in mail.iter() {
            for Msg { port, value } in y_bag {
                if let Some(mdl) = eoc.get(model_name) {
                    if let Some(out_prt) = mdl.get(port) {
                        for out_port in out_prt.iter() {
                            let port = out_port.clone();
                            let value = value.clone();
                            bag.push(Msg { port, value });
                        }
                    }
                }
            }
        }
        bag
    }

    pub(crate) fn init(&mut self, init_time: Time, rng: &mut StdRng) {
        self.dynamic.init(init_time, rng);
    }

    pub(crate) fn time_advance(&self, rng: &mut StdRng) -> Time {
        self.dynamic.time_advance(rng)
    }

    pub(crate) fn state(&self) -> Value {
        self.dynamic.state()
    }

    pub(crate) fn output(&self, sim_time: Time) -> Bag {
        let mut outputs = Outputs::new();
        self.dynamic.output(sim_time, &mut outputs);
        let Outputs { bag } = outputs;
        bag
    }

    pub(crate) fn internal_transition(&mut self, sim_time: Time, rng: &mut StdRng) {
        self.dynamic.internal_transition(sim_time, rng);
    }

    pub(crate) fn external_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
        self.dynamic
            .external_transition(sim_time, elapsed, x_bag, rng)
    }

    pub(crate) fn external_mail_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        mail: &Mail,
        rng: &mut StdRng,
    ) {
        self.dynamic
            .external_mail_transition(sim_time, elapsed, mail, rng)
    }

    pub(crate) fn confluent_transition(&mut self, sim_time: Time, x_bag: &Bag, rng: &mut StdRng) {
        self.dynamic.confluent_transition(sim_time, x_bag, rng);
    }

    pub(crate) fn finish(&mut self, sim_time: Time) -> BTreeMap<String, ModelSimResults> {
        self.dynamic.finish(sim_time);
        let mut results: BTreeMap<String, ModelSimResults> = BTreeMap::new();
        if let Some(sub_simulators) = self.sub_simulators() {
            sub_simulators.iter_mut().for_each(|(_, sub_simulator)| {
                results.append(&mut sub_simulator.finish(sim_time));
            });
        }
        results
    }
}

#[derive(Clone, Debug)]
pub struct Structure {
    pub sub_simulators: BTreeMap<String, Simulator>,
    pub external_input_couplings: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub internal_couplings: BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<String>>>>,
    pub external_output_couplings: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_1() {}
}
