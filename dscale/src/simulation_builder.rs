//! Simulation configuration and builder pattern implementation.
//!
//! This module provides the `SimulationBuilder` struct which uses the builder pattern
//! to configure and construct DScale simulations. It allows you to set up process pools,
//! network topology, bandwidth constraints, timing parameters, and other simulation
//! settings in a fluent, type-safe manner.

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crate::{
    ProcessHandle, ProcessId, Simulation,
    network::BandwidthDescription,
    process_handle::MutableProcessHandle,
    random::Seed,
    time::Jiffies,
    topology::{GLOBAL_POOL, LatencyDescription, LatencyTopology},
};

fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let module_path = record.module_path().unwrap_or("unknown");
            let crate_name = module_path.split("::").next().unwrap_or(module_path);
            use std::io::Write;
            writeln!(buf, "[{}] {}", crate_name, record.args())
        })
        .try_init();
}

/// Builder for configuring and creating DScale simulations.
///
/// `SimulationBuilder` uses the builder pattern to provide a fluent interface for
/// configuring all aspects of a distributed system simulation. It allows you to:
///
/// - Add process pools with different types and sizes
/// - Configure network latency between and within pools
/// - Set bandwidth limitations
/// - Control random seed for deterministic execution
/// - Set simulation time budget
///
/// # Examples
///
/// ```rust
/// use dscale::{SimulationBuilder, Jiffies, BandwidthDescription, LatencyDescription, Distributions};
/// use dscale::ProcessHandle;
///
/// #[derive(Default)]
/// struct MyProcess;
///
/// impl ProcessHandle for MyProcess {
///     fn start(&mut self) {}
///     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
///     fn on_timer(&mut self, id: dscale::TimerId) {}
/// }
///
/// let simulation = SimulationBuilder::default()
///     .seed(12345)
///     .time_budget(Jiffies(1_000_000))
///     .add_pool::<MyProcess>("clients", 3)
///     .add_pool::<MyProcess>("servers", 2)
///     .latency_topology(&[
///         LatencyDescription::WithinPool("servers", Distributions::Uniform(Jiffies(1), Jiffies(3))),
///         LatencyDescription::BetweenPools("clients", "servers", Distributions::Normal(Jiffies(10), Jiffies(2))),
///     ])
///     .nic_bandwidth(BandwidthDescription::Bounded(1000))
///     .build();
///
/// // simulation.run();
/// ```
pub struct SimulationBuilder {
    seed: Seed,
    time_budget: Jiffies,
    proc_id: usize,
    pools: HashMap<String, Vec<(ProcessId, MutableProcessHandle)>>,
    latency_topology: LatencyTopology,
    bandwidth: BandwidthDescription,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        SimulationBuilder {
            seed: 69,
            time_budget: Jiffies(1_000_000),
            proc_id: 1,
            pools: HashMap::new(),
            bandwidth: BandwidthDescription::Unbounded,
            latency_topology: HashMap::new(),
        }
    }
}

impl SimulationBuilder {
    /// Adds a pool of processes of the specified type to the simulation.
    ///
    /// A pool is a named group of processes that all have the same type and behavior.
    /// Pools are useful for organizing different roles in your distributed system
    /// (e.g., "clients", "servers", "coordinators") and for configuring network
    /// topology between different groups.
    ///
    /// Each process in the pool will have a unique [`ProcessId`] and will be
    /// initialized using the [`Default`] trait implementation of the process type.
    ///
    /// # Type Parameters
    ///
    /// * `P` - The process type that implements [`ProcessHandle`] + [`Default`] + `'static`
    ///
    /// # Arguments
    ///
    /// * `name` - A string identifier for the pool (used in topology configuration)
    /// * `size` - The number of processes to create in this pool
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{SimulationBuilder, ProcessHandle, ProcessId, MessagePtr, TimerId};
    ///
    /// #[derive(Default)]
    /// struct Client;
    ///
    /// #[derive(Default)]
    /// struct Server;
    ///
    /// impl ProcessHandle for Client {
    ///     fn start(&mut self) {}
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///     fn on_timer(&mut self, id: TimerId) {}
    /// }
    ///
    /// impl ProcessHandle for Server {
    ///     fn start(&mut self) {}
    ///     fn on_message(&mut self, from: ProcessId, message: MessagePtr) {}
    ///     fn on_timer(&mut self, id: TimerId) {}
    /// }
    ///
    /// let builder = SimulationBuilder::default()
    ///     .add_pool::<Client>("clients", 5)      // 5 client processes
    ///     .add_pool::<Server>("servers", 3);     // 3 server processes
    /// ```
    ///
    /// # Returns
    ///
    /// The `SimulationBuilder` instance for method chaining.
    ///
    /// [`ProcessId`]: crate::ProcessId
    /// [`ProcessHandle`]: crate::ProcessHandle
    pub fn add_pool<P: ProcessHandle + Default + 'static>(
        mut self,
        name: &str,
        size: usize,
    ) -> SimulationBuilder {
        (0..size).for_each(|_| {
            let id = self.proc_id;
            self.proc_id += 1;
            let handle = Rc::new(RefCell::new(P::default()));
            self.add_to_pool::<P>(name, id, handle.clone());
            self.add_to_pool::<P>(GLOBAL_POOL, id, handle.clone());
        });

        self
    }

    fn add_to_pool<P: ProcessHandle + Default + 'static>(
        &mut self,
        name: &str,
        id: usize,
        handle: MutableProcessHandle,
    ) {
        let pool = self.pools.entry(name.to_string()).or_default();
        pool.push((id, handle));
    }

    /// Sets the random seed for deterministic simulation execution.
    ///
    /// The seed controls all random behavior in the simulation, including network
    /// latency generation, random process selection, and any randomness within
    /// your process implementations. Using the same seed with the same configuration
    /// will produce identical simulation results.
    ///
    /// Each process receives a unique seed derived from this base seed to prevent
    /// correlation between processes while maintaining determinism.
    ///
    /// # Arguments
    ///
    /// * `seed` - A `u64` value to use as the base random seed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::SimulationBuilder;
    ///
    /// let builder = SimulationBuilder::default()
    ///     .seed(42);  // Reproducible randomness
    /// ```
    ///
    /// # Returns
    ///
    /// The `SimulationBuilder` instance for method chaining.
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the maximum duration for the simulation.
    ///
    /// The simulation will run until either the specified time budget is reached
    /// or a deadlock is detected (no more events to process). Time is measured
    /// in [`Jiffies`], which are the basic unit of simulation time.
    ///
    /// # Arguments
    ///
    /// * `time_budget` - The maximum simulation time as [`Jiffies`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{SimulationBuilder, Jiffies};
    ///
    /// let builder = SimulationBuilder::default()
    ///     .time_budget(Jiffies(1_000_000));  // Run for 1 million time units
    /// ```
    ///
    /// # Returns
    ///
    /// The `SimulationBuilder` instance for method chaining.
    ///
    /// [`Jiffies`]: crate::Jiffies
    pub fn time_budget(mut self, time_budget: Jiffies) -> Self {
        self.time_budget = time_budget;
        self
    }

    /// Configures network latency between and within process pools.
    ///
    /// This method sets up the network topology by defining latency characteristics
    /// for message delivery between different pools or within the same pool.
    /// Latency is simulated using probability distributions and adds realistic
    /// network delays to message delivery.
    ///
    /// **Important**: This method should be called only after all [`add_pool`] calls
    /// have been made, as it references pool names that must already exist.
    ///
    /// # Arguments
    ///
    /// * `descriptions` - A slice of [`LatencyDescription`] entries defining the topology
    ///
    /// # Latency Types
    ///
    /// - [`LatencyDescription::WithinPool`] - Latency for messages between processes in the same pool
    /// - [`LatencyDescription::BetweenPools`] - Latency for messages between processes in different pools
    ///
    /// # Distribution Types
    ///
    /// - [`Distributions::Uniform`] - Uniform distribution between min and max values
    /// - [`Distributions::Normal`] - Normal (Gaussian) distribution with mean and standard deviation
    /// - [`Distributions::Bernoulli`] - Binary distribution with probability and fixed value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{SimulationBuilder, LatencyDescription, Distributions, Jiffies};
    ///
    /// let builder = SimulationBuilder::default()
    ///     .add_pool::<MyProcess>("clients", 3)
    ///     .add_pool::<MyProcess>("servers", 2)
    ///     .latency_topology(&[
    ///         // Low latency within server pool (1-5 jiffies uniform)
    ///         LatencyDescription::WithinPool("servers",
    ///             Distributions::Uniform(Jiffies(1), Jiffies(5))),
    ///
    ///         // Higher latency between clients and servers (normal distribution)
    ///         LatencyDescription::BetweenPools("clients", "servers",
    ///             Distributions::Normal(Jiffies(50), Jiffies(10))),
    ///
    ///         // Occasional packet loss simulation
    ///         LatencyDescription::WithinPool("clients",
    ///             Distributions::Bernoulli(0.95, Jiffies(20))),
    ///     ]);
    /// # struct MyProcess;
    /// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
    /// # impl dscale::ProcessHandle for MyProcess {
    /// #     fn start(&mut self) {}
    /// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
    /// #     fn on_timer(&mut self, id: dscale::TimerId) {}
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// The `SimulationBuilder` instance for method chaining.
    ///
    /// # Panics
    ///
    /// Panics if a referenced pool name does not exist.
    ///
    /// [`add_pool`]: Self::add_pool
    /// [`LatencyDescription`]: crate::LatencyDescription
    /// [`LatencyDescription::WithinPool`]: crate::LatencyDescription::WithinPool
    /// [`LatencyDescription::BetweenPools`]: crate::LatencyDescription::BetweenPools
    /// [`Distributions::Uniform`]: crate::Distributions::Uniform
    /// [`Distributions::Normal`]: crate::Distributions::Normal
    /// [`Distributions::Bernoulli`]: crate::Distributions::Bernoulli
    pub fn latency_topology(mut self, descriptions: &[LatencyDescription]) -> Self {
        descriptions.iter().for_each(|d| {
            let (from, to, distr) = match d {
                LatencyDescription::WithinPool(name, distr) => (*name, *name, distr),
                LatencyDescription::BetweenPools(pool_from, pool_to, distr) => {
                    (*pool_from, *pool_to, distr)
                }
            };

            let from_vec: Vec<ProcessId> = self
                .pools
                .get(from)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let to_vec: Vec<ProcessId> = self
                .pools
                .get(to)
                .expect("No pool found")
                .iter()
                .map(|(id, _)| *id)
                .collect();

            let cartesian_product = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*x, *y)));

            let cartesian_product_backwards = from_vec
                .iter()
                .flat_map(|x| to_vec.iter().map(move |y| (*y, *x)));

            cartesian_product.for_each(|key| {
                self.latency_topology.insert(key, distr.clone());
            });

            cartesian_product_backwards.for_each(|key| {
                self.latency_topology.insert(key, distr.clone());
            });
        });
        self
    }

    /// Configures network bandwidth limitations for each process.
    ///
    /// This method sets the network interface bandwidth constraints that apply
    /// to each process in the simulation. Bandwidth limits affect how quickly
    /// messages can be transmitted and received, creating realistic network
    /// bottlenecks.
    ///
    /// The bandwidth limit is applied per process (not globally), simulating
    /// individual network interface constraints.
    ///
    /// # Arguments
    ///
    /// * `bandwidth` - A [`BandwidthDescription`] specifying the bandwidth constraints
    ///
    /// # Bandwidth Types
    ///
    /// - [`BandwidthDescription::Unbounded`] - No bandwidth limitations
    /// - [`BandwidthDescription::Bounded(bytes_per_jiffy)`] - Limited to specified bytes per time unit
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::{SimulationBuilder, BandwidthDescription};
    ///
    /// // Unlimited bandwidth
    /// let builder1 = SimulationBuilder::default()
    ///     .nic_bandwidth(BandwidthDescription::Unbounded);
    ///
    /// // Limited to 1000 bytes per jiffy
    /// let builder2 = SimulationBuilder::default()
    ///     .nic_bandwidth(BandwidthDescription::Bounded(1000));
    /// ```
    ///
    /// # Message Sizing
    ///
    /// The bandwidth constraint uses the [`virtual_size()`] method of messages
    /// to determine transmission time. This allows you to simulate large payloads
    /// without actually storing large amounts of data in memory.
    ///
    /// # Returns
    ///
    /// The `SimulationBuilder` instance for method chaining.
    ///
    /// [`BandwidthDescription`]: crate::BandwidthDescription
    /// [`BandwidthDescription::Unbounded`]: crate::BandwidthDescription::Unbounded
    /// [`BandwidthDescription::Bounded`]: crate::BandwidthDescription::Bounded
    /// [`virtual_size()`]: crate::Message::virtual_size
    pub fn nic_bandwidth(mut self, bandwidth: BandwidthDescription) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    /// Finalizes the configuration and builds the simulation.
    ///
    /// This method consumes the `SimulationBuilder` and creates a [`Simulation`]
    /// instance ready to run. It performs final setup including:
    ///
    /// - Initializing the logging system
    /// - Creating the process registry
    /// - Setting up the network topology
    /// - Configuring the simulation engine
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dscale::SimulationBuilder;
    ///
    /// let simulation = SimulationBuilder::default()
    ///     .add_pool::<MyProcess>("nodes", 5)
    ///     .build();
    ///
    /// // simulation.run();
    /// # struct MyProcess;
    /// # impl Default for MyProcess { fn default() -> Self { MyProcess } }
    /// # impl dscale::ProcessHandle for MyProcess {
    /// #     fn start(&mut self) {}
    /// #     fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {}
    /// #     fn on_timer(&mut self, id: dscale::TimerId) {}
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// A configured [`Simulation`] ready to run.
    ///
    /// [`Simulation`]: crate::Simulation
    pub fn build(self) -> Simulation {
        init_logger();

        let mut pool_listing = HashMap::new();
        let mut procs = BTreeMap::new();

        for (name, pool) in self.pools {
            let mut ids = Vec::new();
            for (id, handle) in pool {
                ids.push(id);
                procs.insert(id, handle);
            }
            pool_listing.insert(name, ids);
        }

        Simulation::new(
            self.seed,
            self.time_budget,
            self.bandwidth,
            self.latency_topology,
            pool_listing,
            procs,
        )
    }
}
