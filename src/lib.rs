// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

//! # exdsdevs - Discrete Event System Specification Framework
//!
//! A hierarchical discrete-event simulator implementing the DEVS formalism.
//!
//! ## Key Features
//! - Hierarchical model composition
//! - Parallel execution capabilities
//! - Extensible observer system
//! - Detailed event logging
//! - Stochastic model support
//!
//! ## Basic Usage
//! ```
//! let model = Model::default()
//!     .with_dynamic(MyDynamic)
//!     .with_submodel("child", child_model);
//!
//! let mut experiment = Experiment::new(model, Time::Value(0), Time::Value(100), 10);
//! let results = experiment.run_multi_thread(4);
//! ```

pub mod containers;
pub mod dynamic;
pub mod errors;
pub mod experiment;
pub mod logger;
pub mod model;
pub mod observer;
pub mod root_simulator;
pub mod sim_model;
pub mod simulator;
pub mod time;
pub mod utils;
