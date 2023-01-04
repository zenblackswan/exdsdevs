// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![allow(unused_variables)]
use crate::{
    containers::{Bag, Mail, Value},
    factory::Factory,
    model::Model,
    time::Time,
};

use std::{collections::BTreeMap, marker::PhantomData};

pub trait Observer {
    fn new() -> Self
    where
        Self: Sized;
    fn config(&mut self, observer_config: &Value) {}
    fn init_observer(&mut self, init_config: &Value) {}
    fn on_init(&mut self, model: &Model, init_time: Time, init_value: &Value, t_next: Time) {}
    fn on_outputs(&mut self, model: &Model, sim_time: Time, bag: &Bag) {}
    fn before_internal_transition(&mut self, model: &Model, sim_time: Time) {}
    fn after_internal_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {}
    fn before_external_transition(
        &mut self,
        model: &Model,
        sim_time: Time,
        x_bag: &Bag,
        elapsed: Time,
    ) {
    }
    fn after_external_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {}
    fn before_external_mail_transition(
        &mut self,
        model: &Model,
        sim_time: Time,
        mail: &Mail,
        elapsed: Time,
    ) {
    }
    fn after_external_mail_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {}
    fn before_confluent_transition(&mut self, model: &Model, sim_time: Time, x_bag: &Bag) {}
    fn after_confluent_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {}
    fn after_submodels_transition(&mut self, model: &Model, sim_time: Time, t_next: Time) {}
    fn before_finish(&mut self, model: &Model, sim_time: Time) {}
    fn after_finish(&mut self, model: &Model, sim_time: Time) {}
    fn result(&self) -> Option<Value> {
        None
    }
}

#[derive(Debug, Default)]
pub struct ObserverFactory<T>(PhantomData<T>);
impl<T: Observer> ObserverFactory<T> {
    pub fn new() -> Self {
        ObserverFactory(PhantomData)
    }
}

impl<T: Observer + 'static> Factory for ObserverFactory<T> {
    type Item = Box<dyn Observer>;

    fn create(&self) -> Self::Item {
        Box::new(T::new())
    }
}

unsafe impl<T> Send for ObserverFactory<T> {}
unsafe impl<T> Sync for ObserverFactory<T> {}

#[derive(Default)]
pub struct ObserverFactoryStorage {
    factories: BTreeMap<String, Box<dyn Factory<Item = Box<dyn Observer>>>>,
}

impl ObserverFactoryStorage {
    pub fn new() -> Self {
        Default::default()
    }

    fn add_observer_factory<T: Observer + 'static>(
        &mut self,
        observer_class_name: &str,
        observer_factory: ObserverFactory<T>,
    ) {
        self.factories
            .insert(observer_class_name.to_owned(), Box::new(observer_factory));
    }

    pub fn with_observer_factory<T: Observer + 'static>(
        mut self,
        observer_class_name: &str,
        observer_factory: ObserverFactory<T>,
    ) -> Self {
        self.add_observer_factory(observer_class_name, observer_factory);
        self
    }

    pub fn get_observer(&self, observer_class: &str) -> Result<Box<dyn Observer>, String> {
        if let Some(observer_factory) = self.factories.get(observer_class) {
            Ok(observer_factory.create())
        } else {
            Err(format!(
                "The Observer '{}' have not been registered",
                observer_class
            ))
        }
    }
}
