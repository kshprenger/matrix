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
use matrix::{ProcessHandle, ProcessId, MessagePtr, TimerId, Jiffies};
use matrix::{Broadcast, SendTo, ScheduleTimerAfter, CurrentId, Debug};
use matrix::global::configuration;

#[derive(Default)]
struct MyProcess;

impl ProcessHandle for MyProcess {
    fn Bootstrap(&mut self) {
        Debug!("Starting process {} of {}", CurrentId(), configuration::ProcessNumber());
        // Schedule initial events or broadcast messages
        ScheduleTimerAfter(Jiffies(100));
    }

    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(msg) = message.TryAs::<MyMessage>() {
            Debug!("Received message from {}: {}", from, msg.data);
        }
    }

    fn OnTimer(&mut self, _id: TimerId) {
        // Handle timeouts
        Broadcast(MyMessage { data: 42 });
    }
}
```

### 3. Run the Simulation

Use `SimulationBuilder` to configure and start the simulation.

```rust
use matrix::{SimulationBuilder, Jiffies, BandwidthType};

fn main() {
    let simulation = SimulationBuilder::NewDefault()
        .AddPool::<MyProcess>("PoolName", 4)
        .NICBandwidth(BandwidthType::Unbounded)
        .MaxLatency(Jiffies(10))
        .TimeBudget(Jiffies(1_000_000))
        .Build();

    simulation.Run();
}
```

## Public API

### Simulation Control

- **`SimulationBuilder`**: Configures the simulation environment.
  - `NewDefault()`: Creates simulation with no processes and with default params.
  - `Seed(u64)`: Sets the random seed for deterministic execution.
  - `TimeBudget(Jiffies)`: Sets the maximum duration of the simulation.
  - `MaxLatency(Jiffies)`: Sets the maximum network latency.
  - `AddPool<P: ProcessHandle + Default + 'static>(&str, usize)`: Creates pool of processes with specified name and size.
  - `NICBandwidth(BandwidthType)`: Configures network bandwidth limits.
    - `Bounded(usize)`
    - Or `Unbounded`
  - `Build() -> Simulation`: Clears global vars and builds simulation
- **`Simulation`**: The engine driving the event loop.
  - `Run()`: Starts the simulation loop.

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step (e.g., inside `OnMessage`, `Bootstrap`, or `OnTimer`).

- **`Broadcast(impl Message)`**: Sends a message to all other processes.
- **`BroadcastWithinPool(pool,impl Message)`**: Sends a message to all other processes withing specified pool.
- **`SendTo(ProcessId, impl Message)`**: Sends a message to a specific process.
- **`ScheduleTimerAfter(Jiffies) -> TimerId`**: Schedules a timer interrupt for the current process after a delay.
- **`CurrentId() -> ProcessId`**: Returns the ID of the currently executing process.
- **`Now() -> Jiffies`**: Current time.
- **`ListPool(&str) -> Vec<ProcessId>`**: List all processes that are in the pool with specified name. Panics if pool does not exist.
- **`GlobalUniqueId() -> usize`**: Generates globally-unique id.

### Configuration (`matrix::global::configuration`)

- **`Seed() -> u64`**: Returns the specific seed for the current process (derived from global seed and process ID).
- **`ProcessNumber() -> usize`**: Returns total number of processes in the simulation.

### Any Key-Value (`matrix::global::anykv`)

Useful for passing values, metrics, or shared state between processes or back to the host.

- **`Get<T>(&str) -> T`**
- **`Set<T>(&str, T)`**
- **`Modify<T>(&str, impl FnOnce(&mut T))`**: Modify in-place.

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
- **`RUST_LOG=pingpong=debug`**:
  - Filter events: only `Debug!` macro enabled for user crate (replace `pingpong` with your crate name).

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
