# Distributed system simulator

This project provides a fast & deterministic simulation framework for testing and benchmarking distributed systems. It simulates network latency, bandwidth constraints, and process execution in a single-threaded, event-driven environment.

## Usage

To use the simulator, you need to implement the `ProcessHandle` trait for your distributed system and the `Message` trait for the data exchanged between processes.

### 1. Define Messages

Messages must implement the `Message` trait, which requires defining a `VirtualSize` for bandwidth simulation.

```rust
use simulator::{Message, Jiffies};

struct MyMessage {
    data: u32,
}

impl Message for MyMessage {
    fn VirtualSize(&self) -> usize {
        // Much bigger than real size, but zero-cost
        1000
    }
}
```

### 2. Implement Process Logic

Implement `ProcessHandle` to define how your process reacts to initialization, messages, and timers.

```rust
use simulator::{ProcessHandle, Configuration, ProcessId, MessagePtr, TimerId};
use simulator::{Broadcast, SendTo, ScheduleTimerAfter, CurrentId, Debug};

struct MyProcess;

impl ProcessHandle for MyProcess {
    fn Bootstrap(&mut self, config: Configuration) {
        Debug!("Starting process {}", config.proc_num);
        // Schedule initial events or broadcast messages
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(msg) = message.TryAs::<MyMessage>() {
            Debug!("Received message from {}: {}", from, msg.data);
        }
    }

    fn OnTimer(&mut self, id: TimerId) {
        // Handle timeouts
    }
}
```

### 3. Run the Simulation

Use `SimulationBuilder` to configure and start the simulation.

```rust
use simulator::SimulationBuilder;

fn main() {
    let mut simulation = SimulationBuilder::NewFromFactory(|| Box::new(MyProcess))
        .ProcessInstances(4)
        .Build();

    simulation.Run();
}
```

## Public API

### Simulation Control

- **`SimulationBuilder`**: Configures the simulation environment.
  - `NewFromFactory(Fn() -> Box<dyn ProcessHandle>)`: Sets the factory function to create process instances.
  - `Seed(u64)`: Sets the random seed for deterministic execution.
  - `TimeBudget(Jiffies)`: Sets the maximum duration of the simulation.
  - `MaxLatency(Jiffies)`: Sets the maximum network latency.
  - `ProcessInstances(usize)`: Sets the number of nodes in the cluster.
  - `NICBandwidth(BandwidthType)`: Configures network bandwidth limits.
- **`Simulation`**: The engine driving the event loop.
  - `Run()`: Starts the simulation loop.

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step (e.g., inside `OnMessage`, `Bootstrap`, or `OnTimer`).

- **`Broadcast(impl Message)`**: Sends a message to all other processes.
- **`SendTo(ProcessId, impl Message)`**: Sends a message to a specific process.
- **`ScheduleTimerAfter(Jiffies) -> TimerId`**: Schedules a timer interrupt for the current process after a delay.
- **`CurrentId() -> ProcessId`**: Returns the ID of the currently executing process.

### Logging & Debugging

The simulator integrates with the `log` crate and `env_logger`.

- **`Debug!(fmt, ...)`**: A macro wrapper around `log::debug!` that automatically prepends the current simulation time and process ID.

Debug builds (without the `--release` flag) additionally enable monotonous time-tracking.

## Logging Configuration (`RUST_LOG`)

The simulator output is controlled via the `RUST_LOG` environment variable.

- **`RUST_LOG=info`**:
  - Shows high-level simulation status.
  - Progress bar is enabled
- **`RUST_LOG=debug`**:
  - Enables the `Debug!` macro output from within processes.
  - Useful for tracing message flows and internal state changes.
- **`RUST_LOG=error`**:
  - Shows critical failures.

Example run:

```bash
RUST_LOG=info cargo run --bin example --release
```
