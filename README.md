# Matrix

This project provides a fast & deterministic simulation framework for testing and benchmarking distributed systems. It simulates network latency, bandwidth constraints, and process execution in a single-threaded, event-driven environment.

## Usage

To use the matrix, you need to implement the `ProcessHandle` trait for your distributed system and the `Message` trait for the data exchanged between processes.

### 1. Define Messages

Messages must implement the `Message` trait, which requires defining a `VirtualSize` for bandwidth simulation.

```rust
use matrix::Message;

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
use matrix::{ProcessHandle, Configuration, ProcessId, MessagePtr, TimerId};
use matrix::{Broadcast, SendTo, ScheduleTimerAfter, CurrentId, Debug};

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
use matrix::SimulationBuilder;

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
    - `Bounded(usize)`
    - Or `Unbounded`
- **`Simulation`**: The engine driving the event loop.
  - `Run()`: Starts the simulation loop.

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step (e.g., inside `OnMessage`, `Bootstrap`, or `OnTimer`).

- **`Broadcast(impl Message)`**: Sends a message to all other processes.
- **`SendTo(ProcessId, impl Message)`**: Sends a message to a specific process.
- **`ScheduleTimerAfter(Jiffies) -> TimerId`**: Schedules a timer interrupt for the current process after a delay.
- **`CurrentId() -> ProcessId`**: Returns the ID of the currently executing process.
- **`Now() -> Jiffies`**: Current time.

### Logging & Debugging

Matrix integrates with the `log` crate and `env_logger`.

- **`Debug!(fmt, ...)`**: A macro wrapper around `log::debug!` that automatically prepends current simulation time and process ID.

Debug builds (without the `--release` flag) additionally enable monotonous time-tracking.

## Logging Configuration (`RUST_LOG`)

Matrix output is controlled via the `RUST_LOG` environment variable.

- **`RUST_LOG=info`**:
  - Shows high-level simulation status
  - Progress bar is enabled
- **`RUST_LOG=debug`**:
  - Enables the `Debug!` macro output from within processes.
  - Shows crucial event timepoints during execution
  - In order to filter events use the same variable (read docs for log crate)

Example run:

```bash
RUST_LOG=info cargo run --bin pingpong --release            // With progress bar
RUST_LOG=debug cargo run --bin pingpong --release           // All debug messages
RUST_LOG=pingpong=debug cargo run --bin pingpong --release  // Only Debug! macro enabled for user crate
RUST_LOG=matrix=debug cargo run --bin pingpong --release    // All debug messages for matrix crate
```

## Thanks to

- https://gitlab.com/whirl-framework
- https://github.com/jepsen-io/maelstrom
