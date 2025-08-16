// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

/// Data containers for message passing
use serde_json::Map;
use std::{collections::BTreeMap, rc::Rc};

/// Message container type
pub type Bag = Vec<Msg>;
pub type Mail = Vec<MailItem>;
pub type Value = serde_json::Value;

pub struct Outputs {
    pub(crate) bag: Bag,
}

impl Outputs {
    pub(crate) fn new() -> Self {
        Outputs {
            bag: Default::default(),
        }
    }

    pub fn put(&mut self, port: &str, value: Value) {
        self.bag.push(Msg {
            port: port.to_owned(),
            value: Rc::new(value),
        });
    }
}

/// Inter-model mail item
#[derive(Debug, Clone)]
pub struct MailItem {
    /// Sender model name
    pub model_name: String,
    /// Collection of output messages
    pub y_bag: Bag,
}

/// Message passed between components
#[derive(Debug, Clone)]
pub struct Msg {
    /// Destination port name
    pub(crate) port: String,
    /// Message payload
    pub(crate) value: Rc<Value>,
}

impl Msg {
    /// Creates a new message
    /// # Arguments
    /// - `port`: Destination port name
    /// - `value`: Message payload (JSON-compatible)
    pub fn new(port: &str, value: Value) -> Self {
        Self {
            port: port.to_owned(),
            value: Rc::new(value),
        }
    }

    /// Returns message payload
    pub fn value(&self) -> &Value {
        &*self.value
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

pub struct SimResult {
    pub tags: Vec<String>,
    pub result: Value,
}

pub type ModelSimResults = BTreeMap<String, SimResult>;

impl From<&MailItem> for Value {
    fn from(mail: &MailItem) -> Self {
        let msg_vec: Vec<Value> = mail.y_bag.iter().map(Value::from).collect();
        let mut bag_map = Map::new();
        bag_map.insert(mail.model_name.clone(), Value::Array(msg_vec));
        Value::Object(bag_map)
    }
}

impl From<&Msg> for Value {
    fn from(msg: &Msg) -> Self {
        let mut val_map = Map::new();
        val_map.insert("PORT".to_owned(), Value::String(msg.port.to_owned()));
        val_map.insert("VALUE".to_owned(), msg.value.as_ref().to_owned());
        Value::Object(val_map)
    }
}
