# DScale

This project provides a fast & deterministic simulation framework for testing and benchmarking distributed systems. It simulates network latency, bandwidth constraints, and process execution in a single-threaded, event-driven environment.

## Usage

To use the DScale, you need to implement the `ProcessHandle` trait for your distributed system and the `Message` trait for the data exchanged between processes.

### 1. Install

```shell
cargo add dscale
```

### 2. Define Messages

Messages must implement the `Message` trait, which allows defining a `virtual_size` for bandwidth simulation.

```rust
use dscale::Message;

struct MyMessage {
    data: u32,
}

impl Message for MyMessage {
    fn virtual_size(&self) -> usize {
        // Size in bytes used for bandwidth simulation.
        // Can be much bigger than real memory size to simulate heavy payloads.
        1000
    }
}
```

### 3. Implement Process Logic

Implement `ProcessHandle` to define how your process reacts to initialization, messages, and timers.

```rust
use dscale::{ProcessHandle, ProcessId, MessagePtr, TimerId, Jiffies};
use dscale::{broadcasst, send_to, schedule_timer_after, rank, debug_process};
use dscale::global::configuration;

#[derive(Default)]
struct MyProcess;

impl ProcessHandle for MyProcess {
    fn start(&mut self) {
        debug_process!("Starting process {} of {}", rank(), configuration::process_number());
        // Schedule initial messages or timers
        schedule_timer_after(Jiffies(100));
    }

    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        if let Some(msg) = message.try_as::<MyMessage>() {
            debug_process!("Received message from {}: {}", from, msg.data);
        }
    }

    fn on_timer(&mut self, _id: TimerId) {
        // Handle timeouts
        broadcasst(MyMessage { data: 42 });
    }
}
```

### 4. Run the Simulation

Use `Simulationbuilder` to configure the topology, network constraints, and start the simulation.

```rust
use dscale::{Simulationbuilder, Jiffies, BandwidthDescription, LatencyDescription, Distributions};

fn main() {
    let simulation = SimulationBuilder::default()
        .add_pool::<MyProcess>("Client", 1)
        .add_pool::<MyProcess>("Server", 3)
        .latency_topology(&[
            LatencyDescription::WithinPool("Server", Distributions::Uniform(Jiffies(1), Jiffies(5))),
            LatencyDescription::BetweenPools("Client", "Server", Distributions::Normal(Jiffies(10), Jiffies(2))),
        ])
        .nic_bandwidth(BandwidthDescription::Bounded(1000)) // 1000 bytes per Jiffy
        .time_budget(Jiffies(1_000_000))
        .build();

    simulation.run();
}
```

## Public API

### Simulation Control

- **`SimulationBuilder`**: Configures the simulation environment.
  - `default`: Creates simulation with no processes and default parameters.
  - `seed`: Sets the random seed for deterministic execution.
  - `time_budget`: Sets the maximum duration of the simulation.
  - `add_pool`: Creates a pool of processes. (At the same time all procs become part of GLOBAL_POOL)
  - `latency_topology`: Configures network latency between pools or within them.
  - `nic_bandwidth`: Configures network bandwidth limits (per process).
    - `Bounded`: Limits bandwidth (bytes per jiffy).
    - `Unbounded`: No bandwidth limits.
  - `build`: Finalizes configuration and builds the simulation engine.
- **`Simulation`**: The engine driving the event loop.
  - `run`: Starts the simulation loop.

### Network Topology

- **`GLOBAL_POOL`**:
  - Contains all processes. Simple broadcast uses this pool
- **`LatencyDescription`**:
  - `WithinPool`: Latency for messages between processes in the same pool.
  - `BetweenPools`: Latency for messages between processes in different pools.
- **`Distributions`**:
  - `Uniform`
  - `Bernoulli`
  - `Normal`

### Process Interaction (Context-Aware)

These functions are available globally but must be called within the context of a running process step.

- **`broadcasst`**: Sends a message to all other processes. (GLOBAL_POOL)
- **`broadcasst_within_pool`**: Sends a message to all other processes within a specific pool.
- **`send_to`**: Sends a message to a specific process.
- **`send_random`**: Sends a message to random process. (from GLOBAL_POOL)
- **`send_random_from_pool`**: Sends a message to random process within specific pool.
- **`schedule_timer_after`**: Schedules a timer interrupt for the current process.
- **`rank`**: Returns the ID of the currently executing process.
- **`now`**: Returns current simulation time.
- **`list_pool`**: List all processes in a pool.
- **`choose_from_pool`**: Choose random process id from specified pool.
- **`global_unique_id`**: Generates a globally unique ID.

### Configuration (`dscale::global::configuration`)

- **`seed`**: Returns the specific seed for the current process.
- **`process_number`**: Returns total number of processes in the simulation.

### Any Key-Value (`dscale::global::anykv`)

Useful for passing shared state, metrics, or configuration between processes or back to the host.

- **`get -> T`**
- **`set(T)`**
- **`modify`**: Modify in-place.

### Helpers (`dscale::helpers`)

- **`debug_process!`**: A macro that automatically prepends current simulation time and process ID.
- **`Combiner`**: Structure which allows combining any values up to some known threshols. Can be useful for waiting for quorums.

## Logging Configuration (`RUST_LOG`)

DScale output is controlled via the `RUST_LOG` environment variable.

- **`RUST_LOG=info`**: Shows high-level simulation status and a progress bar.
- **`RUST_LOG=debug`**: Enables all `debug_process!` macro output and all internal simulation events.
- **`RUST_LOG=full::path::to::your::file::or::crate=debug`**: Filter events only for your specific file or crate.

- Note `RUST_LOG=debug or RUST_LOG=any::path=debug` will work only without the `--release` flag.

## Thanks to

- https://gitlab.com/whirl-framework
- https://github.com/jepsen-io/maelstrom
