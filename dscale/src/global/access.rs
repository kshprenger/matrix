use std::{cell::RefCell, rc::Rc};

use crate::{
    Destination, Message, ProcessId,
    actor::EventSubmitter,
    network::NetworkActor,
    random::Randomizer,
    time::{
        Jiffies,
        timer_manager::{NextTimerId, TimerId, TimerManagerActor},
    },
    topology::Topology,
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
    pub(crate) fn New(
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

fn DrainTo<T: EventSubmitter>(submitter: &Rc<RefCell<T>>, events: &mut Vec<T::Event>) {
    if !events.is_empty() {
        submitter.borrow_mut().Submit(events);
    }
}

impl SimulationAccess {
    fn ListPool(&mut self, name: &str) -> &[ProcessId] {
        self.topology.ListPool(name)
    }

    fn ChooseFromPool(&mut self, name: &str) -> ProcessId {
        self.random.ChooseFromSlice(&self.topology.ListPool(name))
    }

    fn BroadcastWithinPool(&mut self, pool_name: &'static str, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::BroadcastWithinPool(pool_name),
            Rc::new(message),
        ));
    }

    fn Broadcast(&mut self, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::Broadcast,
            Rc::new(message),
        ));
    }

    fn SendTo(&mut self, to: ProcessId, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::To(to),
            Rc::new(message),
        ));
    }

    fn SendRandomFromPool(&mut self, pool: &str, message: impl Message + 'static) {
        let target = self.ChooseFromPool(pool);
        self.SendTo(target, message);
    }

    fn ScheduleTimerAfter(&mut self, after: Jiffies) -> TimerId {
        let timer_id = NextTimerId();
        self.scheduled_timers
            .push((self.process_on_execution, timer_id, after));
        timer_id
    }

    fn Drain(&mut self) {
        DrainTo(&self.network, &mut self.scheduled_messages);
        DrainTo(&self.timers, &mut self.scheduled_timers);
    }

    fn SetProcess(&mut self, id: ProcessId) {
        self.process_on_execution = id
    }

    fn Rank(&self) -> ProcessId {
        self.process_on_execution
    }
}

// Any actor makes step -> Buffering outcoming events -> Drain them to all actors
// Before any process step actor should ensure corrent ProcessId on execution via access::SetProcess()
thread_local! {
    pub(crate) static ACCESS_HANDLE: RefCell<Option<SimulationAccess>> = RefCell::new(None);
}

pub(crate) fn Drop() {
    ACCESS_HANDLE.take();
}

pub(crate) fn SetupAccess(
    network: NetworkActor,
    timers: TimerManagerActor,
    topology: Rc<Topology>,
    random: Randomizer,
) {
    ACCESS_HANDLE.with_borrow_mut(|access| {
        *access = Some(SimulationAccess::New(network, timers, topology, random))
    });
}

pub(crate) fn WithAccess<F, T>(f: F) -> T
where
    F: FnOnce(&mut SimulationAccess) -> T,
{
    ACCESS_HANDLE.with_borrow_mut(|access| f(access.as_mut().expect("Out of simulation context")))
}

pub(crate) fn SetProcess(id: ProcessId) {
    WithAccess(|access| access.SetProcess(id));
}

pub(crate) fn Schedule() {
    WithAccess(|access| access.Drain());
}

pub fn ScheduleTimerAfter(after: Jiffies) -> TimerId {
    WithAccess(|access| access.ScheduleTimerAfter(after))
}

pub fn Broadcast(message: impl Message + 'static) {
    WithAccess(|access| access.Broadcast(message));
}

pub fn BroadcastWithinPool(pool: &'static str, message: impl Message + 'static) {
    WithAccess(|access| access.BroadcastWithinPool(pool, message));
}

pub fn SendTo(to: ProcessId, message: impl Message + 'static) {
    WithAccess(|access| access.SendTo(to, message));
}

pub fn SendRandomFromPool(pool: &'static str, message: impl Message + 'static) {
    WithAccess(|access| access.SendRandomFromPool(pool, message));
}

pub fn Rank() -> ProcessId {
    WithAccess(|access| access.Rank())
}

pub fn ListPool(name: &str) -> Vec<ProcessId> {
    WithAccess(|access| access.ListPool(name).to_vec())
}

pub fn ChooseFromPool(name: &str) -> ProcessId {
    WithAccess(|access| access.ChooseFromPool(name))
}
