use std::cell::RefCell;

use crate::{MessagePtr, time::timer_manager::TimerId};

pub type ProcessId = usize;

pub(crate) type UniqueProcessHandle = Box<dyn ProcessHandle>;
pub(crate) type MutableProcessHandle = RefCell<UniqueProcessHandle>;

pub trait ProcessHandle {
    // This method requires process to schedule some initial messages.
    fn start(&mut self);

    // Deliver message
    fn on_message(&mut self, from: ProcessId, message: MessagePtr);

    // Fire timer with id that was returned on ScheduleTimerAfter() call
    fn on_timer(&mut self, id: TimerId);
}
