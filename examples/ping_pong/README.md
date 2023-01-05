# Ping_pong experiment

Models of the agent type are connected in a ring (the output of the previous model is connected to the input of the next model).

Some models have an initial state of **STRIKE**, others have an initial state of WAITING.

During an internal transition, models that are in the **STRIKE** state emit a message ("ball") with the number of the strike made on this "ball".

During an external transition, models that are in the **WAITING** state go into the STRIKE state for a random time interval from 2 to 10.

The Observer writes the history of its transitions and states to the files of each model.

To run the model, enter:

```rust
    cargo run --example ping_pong -- {mode} {path/to/experiment}
```

Where:

**mode** - `single` for run in single thread, `milti` for run in multithred

**path/to/experiment** - relative or absolute path to experiment config file (JSON)

Example

```rust
    cargo run --example ping_pong -- multi experiment examples/ping_pong/experiment_1.json
```
