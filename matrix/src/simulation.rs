use std::{cell::RefCell, collections::HashMap, process::exit, rc::Rc};

use log::{error, info};

use crate::{
    actor::SharedActor,
    global,
    network::{BandwidthType, Network},
    process::{ProcessId, ProcessPool, UniqueProcessHandle},
    progress::Bar,
    random::{self},
    time::{Jiffies, timer_manager::TimerManager},
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
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        pools: HashMap<String, Vec<(ProcessId, UniqueProcessHandle)>>,
    ) -> Self {
        let mut pool_listing = HashMap::new();
        let mut procs = Vec::new();

        for (name, pool) in pools {
            let mut ids = Vec::new();
            for (id, handle) in pool {
                ids.push(id);
                procs.push((id, handle));
            }
            pool_listing.insert(name, ids);
        }

        let proc_pool = ProcessPool::NewShared(procs, pool_listing.clone());

        let network_actor = Rc::new(RefCell::new(Network::New(
            seed,
            max_network_latency,
            bandwidth_type,
            proc_pool.clone(),
        )));

        let timers_actor = Rc::new(RefCell::new(TimerManager::New(proc_pool.clone())));

        global::configuration::SetupGlobalConfiguration(proc_pool.Size());
        global::SetupAccess(network_actor.clone(), timers_actor.clone(), pool_listing);

        let actors = vec![network_actor as SharedActor, timers_actor as SharedActor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::New(time_budget),
        }
    }

    pub fn Run(mut self) {
        self.Start();

        while global::Now() < self.time_budget {
            self.Step();
        }

        // For small simulations progress bar is not fullfilling
        self.progress_bar.MakeProgress(self.time_budget);

        info!("Looks good! ヽ(‘ー`)ノ");
    }
}

impl Simulation {
    fn Start(&mut self) {
        self.actors.iter_mut().for_each(|actor| {
            actor.borrow_mut().Start();
            global::Drain(); // Only after Start() to avoid double borrow_mut() of SharedActor
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
                global::Drain(); // Only after Step() to avoid double borrow_mut() of SharedActor
                self.progress_bar.MakeProgress(future.min(self.time_budget));
            }
        }
    }

    fn PeekClosest(&mut self) -> Option<(Jiffies, SharedActor)> {
        self.actors
            .iter_mut()
            .filter_map(|actor| Some((actor.borrow().PeekClosest()?, actor.clone())))
            .min_by_key(|tuple| tuple.0)
    }
}
