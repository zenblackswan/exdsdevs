// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

/// Time representation module
pub use std::{
    collections::HashSet,
    fmt::Display,
    ops::{Add, Sub},
    rc::Rc,
};
use std::{convert::TryFrom, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{containers::Value, errors::ExdsdevsError};

type Inner = i64;

/// Simulation time value
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Time {
    /// Concrete time value
    Value(Inner),
    /// Infinite time (passive state)
    Inf,
    /// Simulation stop signal
    StopSim,
}

impl Time {
    /// Creates a new time value
    pub fn new(value: Inner) -> Self {
        Time::Value(value)
    }

    /// Infinite time constant
    pub fn inf() -> Self {
        Time::Inf
    }

    pub fn stop() -> Self {
        Time::StopSim
    }
}

impl From<&Time> for Value {
    fn from(time: &Time) -> Self {
        match time {
            Time::Inf => Value::String("Inf".to_owned()),
            Time::Value(value) => Value::Number(From::from(*value)),
            Time::StopSim => Value::String("StopSim".to_owned()),
        }
    }
}

impl TryFrom<&str> for Time {
    type Error = ExdsdevsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Inf" => Ok(Time::Inf),
            "StopSim" => Ok(Time::StopSim),
            time @ _ => Ok(Time::Value(time.parse::<Inner>()?)),
        }
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), std::cmp::Ordering::Equal)
    }
}

impl Eq for Time {}

impl Ord for Time {
    /// Compares time values
    /// # Example
    /// ```
    /// let t1 = Time::Value(5);
    /// let t2 = Time::Value(10);
    /// assert!(t1 < t2);
    /// assert!(Time::Inf > t2);
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Value(left), Self::Value(right)) => left.cmp(right),
            (Self::Inf, Self::Value(_)) => std::cmp::Ordering::Greater,
            (Self::Value(_), Self::Inf) => std::cmp::Ordering::Less,
            (Self::Inf, Self::Inf) => std::cmp::Ordering::Equal,
            (Self::StopSim, _) => std::cmp::Ordering::Less,
            (_, Self::StopSim) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Time {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result {
        match self {
            Self::Inf => write!(f, "Inf"),
            Self::Value(value) => write!(f, "{}", value),
            Self::StopSim => write!(f, "StopSim"),
        }
    }
}

impl Debug for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Inf, _) => Self::Inf,
            (_, Self::Inf) => Self::Inf,
            (Self::Value(left), Self::Value(right)) => Self::Value(left + right),
            (Self::StopSim, Self::StopSim) => Self::StopSim,
            (Self::Value(val), Self::StopSim) => Self::Value(val),
            (Self::StopSim, Self::Value(val)) => Self::Value(val),
        }
    }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Inf, _) => Self::Inf,
            (_, Self::Inf) => Self::Inf,
            (Self::Value(left), Self::Value(right)) => Self::Value(left - right),
            (Self::StopSim, Self::StopSim) => Self::StopSim,
            (Self::Value(val), Self::StopSim) => Self::Value(val),
            (Self::StopSim, Self::Value(val)) => Self::Value(val),
        }
    }
}
