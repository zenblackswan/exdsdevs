// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![allow(unused_variables)]
use std::{collections::BTreeMap, marker::PhantomData};

use rand::rngs::StdRng;

use crate::{
    containers::{Bag, Mail, Value},
    factory::Factory,
    model::{Resources, Structure},
    time::Time,
};

#[allow(unused_variables)]
pub trait Dynamic {
    fn new() -> Self
    where
        Self: Sized;

    fn dynamic_type(&self) -> String;

    fn init(
        &mut self,
        model_structure: &mut Structure,
        init_time: Time,
        init_value: &Value,
        resources: &Resources,
        rng: &mut StdRng,
    ) {
    }

    fn internal_transition(
        &mut self,
        model_structure: &mut Structure,
        sim_time: Time,
        rng: &mut StdRng,
    ) {
    }

    fn external_transition(
        &mut self,
        model_structure: &Structure,
        sim_time: Time,
        elapsed: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
    }

    fn external_mail_transition(
        &mut self,
        model_structure: &mut Structure,
        sim_time: Time,
        elapsed: Time,
        mail: &Mail,
        rng: &mut StdRng,
    ) {
    }

    /// # Variants:
    /// `self.internal_transition(model_structure, sim_time);`
    /// 
    /// `self.external_transition(model_structure, sim_time, Time::Value(0), x_bag);`
    /// 
    /// 
    /// or:
    ///
    /// `self.external_transition(model_structure, sim_time, Time::Value(0), x_bag);`
    /// 
    /// `self.internal_transition(model_structure, sim_time);`
    /// 
    fn confluent_transition(
        &mut self,
        model_structure: &mut Structure,
        sim_time: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
        unimplemented!("Simulation reaches confluent_transition, but it is not implemented!")
    }

    fn output(&self, model_structure: &Structure, sim_time: Time) -> Bag {
        Bag::new()
    }

    fn time_advance(&self, model_structure: &Structure, rng: &mut StdRng) -> Time;

    fn state(&self) -> Value;

    fn finish(&self, sim_time: Time) {}
}

#[derive(Debug, Default)]
pub struct DynamicFactory<T>(PhantomData<T>);
impl<T: Dynamic> DynamicFactory<T> {
    pub fn new() -> Self {
        DynamicFactory(PhantomData)
    }
}

impl<T: Dynamic + 'static> Factory for DynamicFactory<T> {
    type Item = Box<dyn Dynamic>;

    fn create(&self) -> Self::Item {
        Box::new(T::new())
    }
}

unsafe impl<T> Send for DynamicFactory<T> {}
unsafe impl<T> Sync for DynamicFactory<T> {}

#[derive(Default)]
pub struct DynamicFactoryStorage {
    factories: BTreeMap<String, Box<dyn Factory<Item = Box<dyn Dynamic>>>>,
}

impl DynamicFactoryStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_dynamic_factory<T: Dynamic + 'static>(
        &mut self,
        dynamic_class_name: &str,
        dynamic_factory: DynamicFactory<T>,
    ) {
        self.factories
            .insert(dynamic_class_name.to_owned(), Box::new(dynamic_factory));
    }

    pub fn get_dynamic(&self, dynamic_class: &str) -> Result<Box<dyn Dynamic>, String> {
        if let Some(dynamic_factory) = self.factories.get(dynamic_class) {
            Ok(dynamic_factory.create())
        } else {
            Err(format!(
                "The Dynamic '{}' have not been registered",
                dynamic_class
            ))
        }
    }
}
