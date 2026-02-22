use std::{cell::RefCell, rc::Rc};

use crate::destination::Destination;
use crate::now;

use crate::{
    Message, ProcessId,
    actor::EventSubmitter,
    debug_process,
    network::NetworkActor,
    random::Randomizer,
    time::{
        Jiffies,
        timer_manager::{TimerId, TimerManagerActor, next_timer_id},
    },
    topology::{GLOBAL_POOL, Topology},
};

pub struct SimulationAccess {
    process_on_execution: ProcessId,
    pub(crate) scheduled_messages: Vec<(ProcessId, Destination, Rc<dyn Message>)>,
    pub(crate) scheduled_timers: Vec<(ProcessId, TimerId, Jiffies)>,
    topology: Rc<Topology>,
    random: Randomizer,
    network: NetworkActor,
    timers: TimerManagerActor,
}

impl SimulationAccess {
    pub(crate) fn new(
        network: NetworkActor,
        timers: TimerManagerActor,
        topology: Rc<Topology>,
        random: Randomizer,
    ) -> Self {
        Self {
            process_on_execution: 0,
            scheduled_timers: Vec::new(),
            scheduled_messages: Vec::new(),
            topology,
            network,
            timers,
            random,
        }
    }
}

fn drain_to<T: EventSubmitter>(submitter: &Rc<RefCell<T>>, events: &mut Vec<T::Event>) {
    if !events.is_empty() {
        submitter.borrow_mut().submit(events);
    }
}

impl SimulationAccess {
    fn list_pool(&mut self, name: &str) -> &[ProcessId] {
        self.topology.list_pool(name)
    }

    fn choose_from_pool(&mut self, name: &str) -> ProcessId {
        self.random
            .choose_from_slice(&self.topology.list_pool(name))
    }

    fn broadcast_within_pool(&mut self, pool_name: &'static str, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::BroadcastWithinPool(pool_name),
            Rc::new(message),
        ));
    }

    fn send_to(&mut self, to: ProcessId, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::To(to),
            Rc::new(message),
        ));
    }

    fn send_random_from_pool(&mut self, pool: &str, message: impl Message + 'static) {
        let target = self.choose_from_pool(pool);
        self.send_to(target, message);
    }

    fn schedule_timer_after(&mut self, after: Jiffies) -> TimerId {
        let timer_id = next_timer_id();
        self.scheduled_timers
            .push((self.process_on_execution, timer_id, after));
        timer_id
    }

    fn drain(&mut self) {
        drain_to(&self.network, &mut self.scheduled_messages);
        drain_to(&self.timers, &mut self.scheduled_timers);
    }

    fn set_process(&mut self, id: ProcessId) {
        self.process_on_execution = id
    }

    fn rank(&self) -> ProcessId {
        self.process_on_execution
    }
}

// Any actor makes step -> Buffering outcoming events -> Drain them to all actors
// Before any process step actor should ensure corrent ProcessId on execution via access::set_process()
thread_local! {
    pub(crate) static ACCESS_HANDLE: RefCell<Option<SimulationAccess>> = RefCell::new(None);
}

pub(crate) fn drop_access() {
    ACCESS_HANDLE.take();
}

pub(crate) fn setup_access(
    network: NetworkActor,
    timers: TimerManagerActor,
    topology: Rc<Topology>,
    random: Randomizer,
) {
    ACCESS_HANDLE.with_borrow_mut(|access| {
        *access = Some(SimulationAccess::new(network, timers, topology, random))
    });
}

pub(crate) fn with_access<F, T>(f: F) -> T
where
    F: FnOnce(&mut SimulationAccess) -> T,
{
    ACCESS_HANDLE.with_borrow_mut(|access| f(access.as_mut().expect("Out of simulation context")))
}

pub(crate) fn set_process(id: ProcessId) {
    with_access(|access| access.set_process(id));
}

pub(crate) fn schedule() {
    with_access(|access| access.drain());
}

pub fn schedule_timer_after(after: Jiffies) -> TimerId {
    debug_process!("Access: scheduling timer after {after}");
    with_access(|access| access.schedule_timer_after(after))
}

pub fn broadcast(message: impl Message + 'static) {
    debug_process!("Access: broadcasting globally");
    with_access(|access| access.broadcast_within_pool(GLOBAL_POOL, message));
}

pub fn broadcast_within_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("Access: broadcasting within: {pool}");
    with_access(|access| access.broadcast_within_pool(pool, message));
}

pub fn send_to(to: ProcessId, message: impl Message + 'static) {
    debug_process!("Access: send to: {to}");
    with_access(|access| access.send_to(to, message));
}

pub fn send_random(message: impl Message + 'static) {
    debug_process!("Access: sending random in GLOBAL_POOL");
    with_access(|access| access.send_random_from_pool(GLOBAL_POOL, message));
}

pub fn send_random_from_pool(pool: &'static str, message: impl Message + 'static) {
    debug_process!("Access: sending random from pool: {pool}");
    with_access(|access| access.send_random_from_pool(pool, message));
}

pub fn rank() -> ProcessId {
    with_access(|access| access.rank())
}

pub fn list_pool(name: &str) -> Vec<ProcessId> {
    debug_process!("Access: listing pool: {name}");
    with_access(|access| access.list_pool(name).to_vec())
}

pub fn choose_from_pool(name: &str) -> ProcessId {
    debug_process!("Access: choosing random from pool: {name}");
    with_access(|access| access.choose_from_pool(name))
}
