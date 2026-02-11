# DScale

This project provides a fast & deterministic simulation framework for testing and benchmarking distributed systems. It simulates network latency, bandwidth constraints, and process execution in a single-threaded, event-driven environment.

## Usage

To use the DScale, you need to implement the `ProcessHandle` trait for your distributed system and the `Message` trait for the data exchanged between processes.

### 1. Define Messages

Messages must implement the `Message` trait, which allows defining a `VirtualSize` for bandwidth simulation.

```rust
use dscale::Message;

struct MyMessage {
    data: u32,
}

impl Message for MyMessage {
    fn VirtualSize(&self) -> usize {
        // Size in bytes used for bandwidth simulation.
        // Can be much bigger than real memory size to simulate heavy payloads.
        1000
    }
}
```

### 2. Implement Process Logic

Implement `ProcessHandle` to define how your process reacts to initialization, messages, and timers.

```rust
use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Jiffies};
use dscale::{Broadcast, SendTo, ScheduleTimerAfter, Rank, Debug};
use dscale::global::configuration;

#[derive(Default)]
struct MyProcess;

impl ProcessHandle for MyProcess {
    fn Start(&mut self) {
        Debug!("Starting process {} of {}", Rank(), configuration::ProcessNumber());
        // Schedule initial messages or timers
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

Use `SimulationBuilder` to configure the topology, network constraints, and start the simulation.

```rust
use dscale::{SimulationBuilder, Jiffies, BandwidthDescription, LatencyDescription, Distributions};

fn main() {
    let simulation = SimulationBuilder::NewDefault()
        .AddPool::<MyProcess>("Client", 1)
        .AddPool::<MyProcess>("Server", 3)
        .LatencyTopology(&[
            LatencyDescription::WithinPool("Server", Distributions::Uniform(Jiffies(1), Jiffies(5))),
            LatencyDescription::BetweenPools("Client", "Server", Distributions::Normal(Jiffies(10), Jiffies(2))),
        ])
        .NICBandwidth(BandwidthDescription::Bounded(1000)) // 1000 bytes per Jiffy
        .TimeBudget(Jiffies(1_000_000))
        .Build();

    simulation.Run();
}
```

## Public API

### Simulation Control

- **`SimulationBuilder`**: Configures the simulation environment.
  - `NewDefault()`: Creates simulation with no processes and default parameters.
  - `Seed(u64)`: Sets the random seed for deterministic execution.
  - `TimeBudget(Jiffies)`: Sets the maximum duration of the simulation.
  - `AddPool<P: ProcessHandle + Default + 'static>(&str, usize)`: Creates a pool of processes.
  - `LatencyTopology(&[LatencyDescription])`: Configures network latency between pools or within them.
  - `NICBandwidth(BandwidthDescription)`: Configures network bandwidth limits (per process).
    - `Bounded(usize)`: Limits bandwidth (bytes per jiffy).
    - `Unbounded`: No bandwidth limits.
  - `Build() -> Simulation`: Finalizes configuration and builds the simulation engine.
- **`Simulation`**: The engine driving the event loop.
  - `Run()`: Starts the simulation loop.

### Network Topology

- **`LatencyDescription`**:
  - `WithinPool(&str, Distributions)`: Latency for messages between processes in the same pool.
  - `BetweenPools(&str, &str, Distributions)`: Latency for messages between processes in different pools.
- **`Distributions`**:
  - `Uniform(Jiffies, Jiffies)`
  - `Bernoulli(f64, Jiffies)`
  - `Normal(Jiffies, Jiffies)`

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step.

- **`Broadcast(impl Message)`**: Sends a message to all other processes.
- **`BroadcastWithinPool(&str, impl Message)`**: Sends a message to all other processes within a specific pool.
- **`SendTo(ProcessId, impl Message)`**: Sends a message to a specific process.
- **`SendRandomFromPool(&str, impl Message)`**: Sends a message to random process whithin pool.
- **`ScheduleTimerAfter(Jiffies) -> TimerId`**: Schedules a timer interrupt for the current process.
- **`Rank() -> ProcessId`**: Returns the ID of the currently executing process.
- **`Now() -> Jiffies`**: Returns current simulation time.
- **`ListPool(&str) -> Vec<ProcessId>`**: List all processes in a pool.
- **`ChooseFromPool(&str) -> ProcessId`**: Choose random process id from specified pool.
- **`GlobalUniqueId() -> usize`**: Generates a globally unique ID.

### Configuration (`dscale::global::configuration`)

- **`Seed() -> u64`**: Returns the specific seed for the current process.
- **`ProcessNumber() -> usize`**: Returns total number of processes in the simulation.

### Any Key-Value (`dscale::global::anykv`)

Useful for passing shared state, metrics, or configuration between processes or back to the host.

- **`Get<T>(&str) -> T`**
- **`Set<T>(&str, T)`**
- **`Modify<T>(&str, impl FnOnce(&mut T))`**: Modify in-place.

### Logging & Debugging

- **`Debug!(fmt, ...)`**: A macro that automatically prepends current simulation time and process ID.

## Logging Configuration (`RUST_LOG`)

DScale output is controlled via the `RUST_LOG` environment variable.

- **`RUST_LOG=info`**: Shows high-level simulation status and a progress bar.
- **`RUST_LOG=debug`**: Enables the `Debug!` macro output and internal simulation events.
- **`RUST_LOG=your_crate=debug`**: Filter events only for your specific crate.

- Note `RUST_LOG=debug or RUST_LOG=any=debug` will work only without the `--release` flag.

## Thanks to

- https://gitlab.com/whirl-framework
- https://github.com/jepsen-io/maelstrom
