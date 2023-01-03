// Copyright 2023 Developers of the exdsdevs project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use serde_json::Map;
use std::rc::Rc;

pub type Bag = Vec<Msg>;
pub type Mail = Vec<MailItem>;
pub type Value = serde_json::Value;

#[derive(Debug, Clone)]
pub struct MailItem {
    pub model_name: String,
    pub y_bag: Bag,
}

#[derive(Debug, Clone)]
pub struct Msg {
    pub port: String,
    pub value: Rc<Value>,
}

impl Msg {
    pub fn new(port: &str, value: Value) -> Self {
        Self {
            port: port.to_owned(),
            value: Rc::new(value),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

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
