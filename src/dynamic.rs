// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

/// Model behavior definition module
use std::fmt::Debug;

use dyn_clone::DynClone;
use rand::rngs::StdRng;

use crate::{
    containers::{Bag, Mail, Outputs, Value},
    time::Time,
};

impl Debug for Box<dyn Dynamic> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.state();
        Debug::fmt(&val, f)
    }
}

impl Default for Box<dyn Dynamic> {
    fn default() -> Self {
        Box::new(DefaultDynamic)
    }
}

/// Trait defining DEVS component behavior
/// # Implementation Example
/// ```
/// #[derive(Clone)]
/// struct MyDynamic {
///     state: u32,
/// }
///
/// impl Dynamic for MyDynamic {
///     fn time_advance(&self) -> Time {
///         Time::Value(1)
///     }
///
///     fn internal_transition(&mut self) {
///         self.state += 1;
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait Dynamic: DynClone + Send {
    /// Model initialization
    fn init(&mut self, init_time: Time, rng: &mut StdRng) {}

    /// Internal state transition
    fn internal_transition(&mut self, sim_time: Time, rng: &mut StdRng) {}

    /// External state transition
    fn external_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
    }

    fn external_mail_transition(
        &mut self,
        sim_time: Time,
        elapsed: Time,
        mail: &Mail,
        rng: &mut StdRng,
    ) {
    }

    /// # Variants:
    /// ```
    /// self.internal_transition(sim_time, rng);
    /// self.external_transition(sim_time, Time::Value(0), x_bag, rng);
    ///```
    ///
    /// or:
    ///```
    /// self.external_transition(sim_time, Time::Value(0), x_bag, rng);
    /// self.internal_transition(sim_time, rng);
    ///```
    fn confluent_transition(&mut self, sim_time: Time, x_bag: &Bag, rng: &mut StdRng) {
        unimplemented!("Simulation reaches confluent_transition, but it is not implemented!")
    }

    /// Output message generation
    fn output(&self, sim_time: Time, outputs: &mut Outputs) {}

    /// Time to next internal event
    fn time_advance(&self, rng: &mut StdRng) -> Time;

    fn state(&self) -> Value;

    fn finish(&self, sim_time: Time) {}
}

impl Clone for Box<dyn Dynamic> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[derive(Clone)]
pub struct DefaultDynamic;

impl Dynamic for DefaultDynamic {
    fn time_advance(&self, _: &mut StdRng) -> Time {
        Time::Inf
    }

    fn state(&self) -> Value {
        Value::Null
    }
}
