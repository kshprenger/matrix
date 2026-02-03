use std::{cell::RefCell, process::exit, rc::Rc, usize};

use log::{error, info};

use crate::{
    actor::SharedActor,
    global,
    network::{BandwidthDescription, Network},
    progress::Bar,
    random::{self, Randomizer},
    time::{Jiffies, timer_manager::TimerManager},
    topology::{HandlerMap, LatencyTopology, PoolListing, Topology},
};

pub struct Simulation {
    actors: Vec<SharedActor>,
    time_budget: Jiffies,
    progress_bar: Bar,
}

impl Simulation {
    pub(crate) fn New(
        seed: random::Seed,
        time_budget: Jiffies,
        bandwidth: BandwidthDescription,
        latency_topology: LatencyTopology,
        pool_listing: PoolListing,
        procs: HandlerMap,
    ) -> Self {
        let topology = Topology::NewShared(procs, pool_listing.clone(), latency_topology);

        let network_actor = Rc::new(RefCell::new(Network::New(
            seed,
            bandwidth,
            topology.clone(),
        )));

        let timers_actor = Rc::new(RefCell::new(TimerManager::New(topology.clone())));

        global::configuration::SetupGlobalConfiguration(topology.Size());
        global::SetupAccess(
            network_actor.clone(),
            timers_actor.clone(),
            topology,
            Randomizer::New(seed),
        );

        let actors: Vec<SharedActor> = vec![network_actor, timers_actor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::New(time_budget),
        }
    }

    pub fn Run(&mut self) {
        self.Start();

        while global::Now() < self.time_budget {
            self.Step();
        }

        // For small simulations progress bar is not fullfilling
        self.progress_bar.Finish();

        info!("Looks good! ヽ(‘ー`)ノ");
    }
}

impl Simulation {
    fn Start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            actor.borrow_mut().Start();
            global::Schedule(); // Only after Start() to avoid double borrow_mut() of SharedActor
        });
    }

    fn Step(&mut self) {
        match self.PeekClosest() {
            None => {
                error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
            Some((future, actor)) => {
                global::FastForwardClock(future);
                actor.borrow_mut().Step();
                global::Schedule(); // Only after Step() to avoid double borrow_mut() of SharedActor
                self.progress_bar.MakeProgress(future.min(self.time_budget));
            }
        }
    }

    fn PeekClosest(&mut self) -> Option<(Jiffies, SharedActor)> {
        let mut min_time = Jiffies(usize::MAX);
        let mut sha: Option<SharedActor> = None;
        for actor in self.actors.iter() {
            actor.borrow().PeekClosest().map(|time| {
                if time < min_time {
                    min_time = time;
                    sha = Some(actor.clone())
                }
            });
        }

        Some((min_time, sha?))
    }
}

impl Drop for Simulation {
    fn drop(&mut self) {
        global::Drop(); // Clear thread_locals
    }
}
