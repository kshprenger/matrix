use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    Destination, Message, ProcessId,
    network::Network,
    time::{
        Jiffies,
        timer_manager::{NextTimerId, TimerId, TimerManager},
    },
};

pub struct SimulationAccess {
    process_on_execution: ProcessId,
    pub(crate) scheduled_messages: Vec<(ProcessId, Destination, Rc<dyn Message>)>,
    pub(crate) scheduled_timers: Vec<(ProcessId, TimerId, Jiffies)>,
    pools: HashMap<String, Vec<ProcessId>>,
    network: Rc<RefCell<Network>>,
    timers: Rc<RefCell<TimerManager>>,
}

impl SimulationAccess {
    pub(crate) fn New(
        network: Rc<RefCell<Network>>,
        timers: Rc<RefCell<TimerManager>>,
        pools: HashMap<String, Vec<ProcessId>>,
    ) -> Self {
        Self {
            process_on_execution: 0,
            scheduled_timers: Vec::new(),
            scheduled_messages: Vec::new(),
            pools,
            network,
            timers,
        }
    }
}

impl SimulationAccess {
    fn ListPool(&mut self, name: &str) -> &[ProcessId] {
        self.pools.get(name).expect("Pool does not exist")
    }

    fn BroadcastWithinPool(&mut self, pool_name: &'static str, message: impl Message + 'static) {
        self.scheduled_messages.push((
            self.process_on_execution,
            Destination::BroadcastWithingPool(pool_name),
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

    fn ScheduleTimerAfter(&mut self, after: Jiffies) -> TimerId {
        let timer_id = NextTimerId();
        self.scheduled_timers
            .push((self.process_on_execution, timer_id, after));
        timer_id
    }

    fn Drain(&mut self) {
        self.network
            .borrow_mut()
            .SubmitMessages(&mut self.scheduled_messages);
        self.timers
            .borrow_mut()
            .ScheduleTimers(&mut self.scheduled_timers);
    }

    fn SetProcess(&mut self, id: ProcessId) {
        self.process_on_execution = id
    }

    fn CurrentId(&self) -> ProcessId {
        self.process_on_execution
    }
}

// Any actor makes step -> Buffering outcoming events -> Drain them to all actors
// Before any process step actor should ensure corrent ProcessId on execution via access::SetProcess()
thread_local! {
    pub(crate) static ACCESS_HANDLE: RefCell<Option<SimulationAccess>> = RefCell::new(None);
}

pub(crate) fn SetupAccess(
    network: Rc<RefCell<Network>>,
    timers: Rc<RefCell<TimerManager>>,
    pools: HashMap<String, Vec<ProcessId>>,
) {
    ACCESS_HANDLE
        .with_borrow_mut(|access| *access = Some(SimulationAccess::New(network, timers, pools)));
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

pub(crate) fn Drain() {
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

pub fn CurrentId() -> ProcessId {
    WithAccess(|access| access.CurrentId())
}

pub fn ListPool(name: &str) -> Vec<ProcessId> {
    WithAccess(|access| access.ListPool(name).to_vec())
}

// Userspace debugger
#[macro_export]
macro_rules! Debug {
    ($($arg:tt)+) => {
        log::debug!("[Now: {} | Process {}] {}", Now(), CurrentId(), format_args!($($arg)+));
    }
}
