use std::{cell::RefCell, process::exit, rc::Rc};

use log::{error, info};

use crate::{
    access,
    actor::SharedActor,
    network::{BandwidthType, Network},
    process::{ProcessId, ProcessPool, UniqueProcessHandle},
    progress::Bar,
    random::{self},
    time::{self, Jiffies, timer::Timers},
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
        procs: Vec<(ProcessId, UniqueProcessHandle)>,
    ) -> Self {
        let proc_pool = ProcessPool::NewShared(procs);

        let network_actor = Rc::new(RefCell::new(Network::New(
            seed,
            max_network_latency,
            bandwidth_type,
            proc_pool.clone(),
        )));

        let timers_actor = Rc::new(RefCell::new(Timers::New(proc_pool.clone())));

        access::SetupAccess(network_actor.clone(), timers_actor.clone());

        let actors = vec![network_actor as SharedActor, timers_actor as SharedActor];

        Self {
            actors,
            time_budget,
            progress_bar: Bar::New(time_budget),
        }
    }

    pub fn Run(mut self) {
        self.Start();

        while time::Now() < self.time_budget {
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
            access::Drain(); // Only after Start() to avoid double borrow_mut() of SharedActor
        });
    }

    fn Step(&mut self) {
        match self.PeekClosest() {
            None => {
                error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
            Some((future, actor)) => {
                time::FastForwardClock(future);
                actor.borrow_mut().Step();
                access::Drain(); // Only after Step() to avoid double borrow_mut() of SharedActor
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
