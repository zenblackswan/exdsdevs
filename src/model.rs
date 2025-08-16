// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

/// Hierarchical model construction
use crate::{dynamic::Dynamic, observer::Observer};
use std::collections::BTreeMap;

/// DEVS model builder
/// # Hierarchical Composition
/// ```
/// let root = Model::default()
///     .with_submodel("child", child_model)
///     .with_input_coupling(("in", "child", "in"))
///     .with_output_coupling(("child", "out", "out"));
/// ```
#[derive(Clone)]
pub struct Model {
    pub(crate) dynamic: Box<dyn Dynamic>,
    pub(crate) input_ports: Vec<String>,
    pub(crate) output_ports: Vec<String>,
    pub(crate) sumbodels: BTreeMap<String, Model>,
    pub(crate) input_couplings: Vec<(String, String, String)>,
    pub(crate) internal_couplings: Vec<(String, String, String, String)>,
    pub(crate) output_couplings: Vec<(String, String, String)>,
    pub(crate) observers: BTreeMap<String, Box<dyn Observer>>,
}

impl Model {
    /// Adds dynamic behavior
    pub fn with_dynamic<T: Dynamic + 'static>(mut self, dynamic: T) -> Self {
        self.dynamic = Box::new(dynamic);
        self
    }

    /// Creates input port coupling
    pub fn with_input_ports(mut self, input_ports: Vec<&str>) -> Self {
        if self.input_ports.is_empty() {
            self.input_ports = input_ports.iter().map(|&p| p.to_owned()).collect();
        } else {
            panic!("ERROR: input ports for model is already set");
        }
        self
    }

    /// Creates output port coupling
    pub fn with_output_ports(mut self, output_ports: Vec<&str>) -> Self {
        if self.output_ports.is_empty() {
            self.output_ports = output_ports.iter().map(|&p| p.to_owned()).collect();
        } else {
            panic!("ERROR: output ports for model is already set");
        }
        self
    }
    /// Adds a submodel
    /// # Arguments
    /// - `submodel_name`: Unique submodel identifier
    /// - `submodel`: Model instance
    pub fn with_submodel(mut self, submodel_name: &str, submodel: Model) -> Self {
        if !self.sumbodels.contains_key(submodel_name) {
            self.sumbodels.insert(submodel_name.to_owned(), submodel);
        } else {
            panic!(
                "ERROR: model already contains submodel with name {}",
                submodel_name
            );
        }
        self
    }

    /// Creates input coupling
    pub fn with_input_coupling(mut self, input_coupling: (&str, &str, &str)) -> Self {
        let (self_input_port, submodel, submodel_input_port) = input_coupling;
        let input_coupling = (
            self_input_port.to_owned(),
            submodel.to_owned(),
            submodel_input_port.to_owned(),
        );
        if !self.input_couplings.contains(&input_coupling) {
            self.input_couplings.push(input_coupling);
        }
        self
    }

    pub fn with_internal_coupling(mut self, internal_coupling: (&str, &str, &str, &str)) -> Self {
        let (source_submodel, source_submodel_output_port, dest_submodel, dest_submodel_input_port) =
            internal_coupling;
        let internal_coupling = (
            source_submodel.to_owned(),
            source_submodel_output_port.to_owned(),
            dest_submodel.to_owned(),
            dest_submodel_input_port.to_owned(),
        );
        if !self.internal_couplings.contains(&internal_coupling) {
            self.internal_couplings.push(internal_coupling);
        }
        self
    }

    pub fn with_output_coupling(mut self, output_coupling: (&str, &str, &str)) -> Self {
        let (submodel, submodel_output_port, self_output_port) = output_coupling;
        let output_coupling = (
            submodel.to_owned(),
            submodel_output_port.to_owned(),
            self_output_port.to_owned(),
        );
        if !self.output_couplings.contains(&output_coupling) {
            self.output_couplings.push(output_coupling);
        }
        self
    }

    pub fn with_observer<T: Observer + 'static>(
        mut self,
        observer_name: &str,
        observer: T,
    ) -> Self {
        self.observers
            .insert(observer_name.to_owned(), Box::new(observer));
        self
    }

    pub fn check(&self) -> Result<(), ()> {
        todo!()
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            dynamic: Default::default(),
            input_ports: Default::default(),
            output_ports: Default::default(),
            sumbodels: Default::default(),
            input_couplings: Default::default(),
            internal_couplings: Default::default(),
            output_couplings: Default::default(),
            observers: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::containers::Value;
    use crate::dynamic::DefaultDynamic;
    use crate::time::Time;
    use rand::rngs::StdRng;

    #[derive(Clone)]
    struct TestDynamic {
        _state: u64,
    }

    impl Dynamic for TestDynamic {
        fn time_advance(&self, _rng: &mut StdRng) -> Time {
            todo!()
        }

        fn state(&self) -> Value {
            todo!()
        }
    }

    #[test]
    fn build_model_ping_pong() {
        let s1_dynamic = TestDynamic { _state: 1 };
        let s2_dynamic = TestDynamic { _state: 0 };
        let s1 = Model {
            dynamic: Box::new(s1_dynamic),
            input_ports: vec![],
            output_ports: vec!["out".to_owned()],
            sumbodels: BTreeMap::new(),
            input_couplings: vec![],
            internal_couplings: vec![],
            output_couplings: vec![],
            observers: Default::default(),
        };

        let s2 = Model {
            dynamic: Box::new(s2_dynamic),
            input_ports: vec![],
            output_ports: vec!["out".to_owned()],
            sumbodels: BTreeMap::new(),
            input_couplings: vec![],
            internal_couplings: vec![],
            output_couplings: vec![],
            observers: Default::default(),
        };

        let _s0 = Model {
            dynamic: Box::new(DefaultDynamic),
            input_ports: Default::default(),
            output_ports: Default::default(),
            sumbodels: [("s1".to_owned(), s1), ("s2".to_owned(), s2)].into(),
            input_couplings: Default::default(),
            internal_couplings: vec![(
                "s1".to_owned(),
                "out".to_owned(),
                "s2".to_owned(),
                "in".to_owned(),
            )],
            output_couplings: Default::default(),
            observers: Default::default(),
        };
    }
}
