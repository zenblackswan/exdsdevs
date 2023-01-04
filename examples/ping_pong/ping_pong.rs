use exdsdevs::{
    containers::{Bag, Msg, Value},
    dynamic::Dynamic,
    model::{Resources, Structure},
    time::Time,
};
use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum State {
    WAITING,
    STRIKE,
}

pub struct AgentDynamic {
    state: State,
    pub last_count: i64,
}

impl Dynamic for AgentDynamic {
    fn new() -> Self {
        Self {
            state: State::WAITING,
            last_count: 0,
        }
    }

    fn init(
        &mut self,
        _: &mut Structure,
        _: Time,
        init_value: &Value,
        _: &Resources,
        _: &mut StdRng,
    ) {
        if let Some(state) = init_value.get("state") {
            self.state = serde_json::from_value::<State>(state.clone())
                .unwrap_or_else(|_| panic!("Unknown state"));
        }
        self.last_count = 0;
    }

    fn internal_transition(
        &mut self,
        _: &mut Structure,
        _sim_time: Time,
        _: &mut StdRng,
    ) {
        self.state = match self.state {
            State::STRIKE => State::WAITING,
            State::WAITING => State::WAITING,
        }
    }

    fn external_transition(
        &mut self,
        _: &Structure,
        _: Time,
        _: Time,
        x_bag: &Bag,
        _: &mut StdRng,
    ) {
        match self.state {
            State::WAITING => {
                self.state = State::STRIKE;

                if let Some(msg) = x_bag.get(0) {
                    if let Value::Number(count) = msg.value() {
                        if let Some(c) = count.as_i64() {
                            self.last_count = c + 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn confluent_transition(
        &mut self,
        model_structure: &mut Structure,
        sim_time: Time,
        x_bag: &Bag,
        rng: &mut StdRng,
    ) {
        self.internal_transition(model_structure, sim_time, rng);
        self.external_transition(model_structure, sim_time, Time::Value(0), x_bag, rng);
    }

    fn output(&self, _atomic_model_structure: &Structure, _sim_time: Time) -> Bag {
        match self.state {
            State::STRIKE => {
                let msg = Msg::new("out", Value::from(self.last_count));
                vec![msg]
            }
            _ => panic!("Unknown state"),
        }
    }

    fn time_advance(&self, _atomic_model_structure: &Structure, rng: &mut StdRng) -> Time {
        match self.state {
            State::STRIKE => Time::Value(rng.gen_range(2..10i128)),
            State::WAITING => Time::Inf,
        }
    }

    fn state(&self) -> Value {
        serde_json::to_value(&self.state).unwrap()
    }

    fn dynamic_type(&self) -> String {
        "agent".to_string()
    }
}

pub struct RootDynamic;

impl Dynamic for RootDynamic {
    fn new() -> Self {
        Self
    }

    fn time_advance(&self, _: &Structure, _: &mut StdRng) -> Time {
        Time::Inf
    }

    fn state(&self) -> Value {
        Value::Null
    }

    fn dynamic_type(&self) -> String {
        "root".to_string()
    }
}
