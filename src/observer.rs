// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

#![allow(unused_variables)]
use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::{
    containers::{Bag, Mail, SimResult},
    sim_model::SimModel,
    time::Time,
};

impl Debug for Box<dyn Observer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt("OBSERVER", f)
    }
}

/// Simulation observation system

/// Trait for simulation event observation
/// # Usage Example
/// ```
/// #[derive(Clone)]
/// struct StatsCollector {
///     event_count: u64,
/// }
///
/// impl Observer for StatsCollector {
///     fn after_internal_transition(&mut self) {
///         self.event_count += 1;
///     }
/// }
/// ```
pub trait Observer: DynClone + Send {
    fn init(&mut self, model: &SimModel, iteration: u64) {}
    /// Called after model initialization
    fn on_init(&mut self, model: &SimModel, init_time: Time, t_next: Time) {}
    fn on_outputs(&mut self, model: &SimModel, sim_time: Time, bag: &Bag) {}
    /// Called before internal transition
    fn before_internal_transition(&mut self, model: &SimModel, sim_time: Time) {}
    /// Called after internal transition
    fn after_internal_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {}
    fn before_external_transition(
        &mut self,
        model: &SimModel,
        sim_time: Time,
        x_bag: &Bag,
        elapsed: Time,
    ) {
    }
    fn after_external_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {}
    fn before_external_mail_transition(
        &mut self,
        model: &SimModel,
        sim_time: Time,
        mail: &Mail,
        elapsed: Time,
    ) {
    }
    fn after_external_mail_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {}
    fn before_confluent_transition(&mut self, model: &SimModel, sim_time: Time, x_bag: &Bag) {}
    fn after_confluent_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {}
    fn after_submodels_transition(&mut self, model: &SimModel, sim_time: Time, t_next: Time) {}
    fn finish(&mut self, model: &SimModel, sim_time: Time) -> Option<SimResult> {
        None
    }
}

impl Clone for Box<dyn Observer> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
