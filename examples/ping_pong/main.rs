use std::env::{args, current_dir};

use exdsdevs::{
    containers::{Bag, Outputs, Value},
    dynamic::Dynamic,
    logger::Logger,
    model::Model,
    root_simulator::RootSimulator,
    time::Time,
};
use rand::{rngs::StdRng, Rng};
use serde::Serialize;
use serde_json::json;

#[derive(Clone, Serialize)]
enum State {
    IDLE,
    ACTIVE,
}

#[derive(Clone, Serialize)]
struct TestDynamic {
    state: State,
    count: u64,
}

impl Dynamic for TestDynamic {
    fn time_advance(&self, rng: &mut StdRng) -> Time {
        match self.state {
            State::IDLE => Time::inf(),
            State::ACTIVE => {
                let ta = (rng.gen::<f64>() * 10.0) as i64;
                Time::new(ta)
            }
        }
    }

    fn internal_transition(&mut self, _: Time, _: &mut StdRng) {
        self.state = match self.state {
            State::IDLE => State::ACTIVE,
            State::ACTIVE => State::IDLE,
        };
    }

    fn external_transition(&mut self, _: Time, _: Time, _: &Bag, _: &mut StdRng) {
        self.state = match self.state {
            State::IDLE => State::ACTIVE,
            State::ACTIVE => State::IDLE,
        };
        self.count += 1;
    }

    fn output(&self, _: Time, outputs: &mut Outputs) {
        match self.state {
            State::IDLE => (),
            State::ACTIVE => outputs.put("out", Value::Null),
        }
    }

    fn state(&self) -> Value {
        json!(self)
    }
}

fn build_model() -> Model {
    let out_dir = current_dir().unwrap().join("examples").join("ping_pong").join("out_dir");
    let s1 = Model::default()
        .with_dynamic(TestDynamic {
            state: State::ACTIVE,
            count: 0,
        })
        .with_input_ports(vec!["in"])
        .with_output_ports(vec!["out"])
        .with_observer("s1_obs", Logger::new(&out_dir));

    let s2 = Model::default()
        .with_dynamic(TestDynamic {
            state: State::IDLE,
            count: 0,
        })
        .with_input_ports(vec!["in"])
        .with_output_ports(vec!["out"])
        .with_observer("s2_obs", Logger::new(&out_dir));

    Model::default()
        .with_submodel("s1", s1)
        .with_submodel("s2", s2)
        .with_internal_coupling(("s1", "out", "s2", "in"))
        .with_internal_coupling(("s2", "out", "s1", "in"))
}

pub fn main() {
    let mut argv = args();
    argv.next();
    // let t: i64 = argv.next().unwrap().parse().unwrap();
    // let random_seed = argv.next().unwrap().parse().unwrap();

    let t: i64 = 100;
    let random_seed = 1;

    let example_model = build_model();
    let mut root_sim = RootSimulator::new(example_model, 1).unwrap();
    root_sim.init(Time::Value(0), Time::Value(t), random_seed);
    root_sim.run();
}
