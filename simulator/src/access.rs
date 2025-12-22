use std::{cell::RefCell, rc::Rc};

use crate::{Destination, Jiffies, Message, ProcessId};

pub struct Access {
    pub(crate) scheduled_messages: Vec<(Destination, Rc<dyn Message>)>,
    pub(crate) current_time: Jiffies,
}

impl Access {
    pub(crate) fn New(current_time: Jiffies) -> Self {
        Self {
            scheduled_messages: Vec::new(),
            current_time,
        }
    }
}

impl Access {
    fn Broadcast(&mut self, message: impl Message + 'static) {
        self.scheduled_messages
            .push((Destination::Broadcast, Rc::new(message)));
    }

    fn SendTo(&mut self, to: ProcessId, message: impl Message + 'static) {
        self.scheduled_messages
            .push((Destination::To(to), Rc::new(message)));
    }

    fn SendSelf(&mut self, message: impl Message + 'static) {
        self.scheduled_messages
            .push((Destination::SendSelf, Rc::new(message)));
    }

    fn Now(&self) -> Jiffies {
        self.current_time
    }

    fn DrainMessages(&mut self) -> Vec<(Destination, Rc<dyn Message>)> {
        self.scheduled_messages.drain(..).collect()
    }
}

thread_local! {
    pub(crate) static ACCESS_HANDLE: RefCell<Option<Access>> = RefCell::new(None);
}

pub(crate) fn SetupAccess(a: Access) {
    ACCESS_HANDLE.with_borrow_mut(|access| *access = Some(a))
}

pub(crate) fn WithAccess<F, T>(f: F) -> T
where
    F: FnOnce(&mut Access) -> T,
{
    ACCESS_HANDLE.with_borrow_mut(|access| f(access.as_mut().expect("Out of simulation context")))
}

pub(crate) fn DrainMessages() -> Vec<(Destination, Rc<dyn Message>)> {
    WithAccess(|access| access.DrainMessages())
}

pub fn Broadcast(message: impl Message + 'static) {
    WithAccess(|access| access.Broadcast(message));
}

pub fn SendTo(to: ProcessId, message: impl Message + 'static) {
    WithAccess(|access| access.SendTo(to, message));
}

pub fn SendSelf(message: impl Message + 'static) {
    WithAccess(|access| access.SendSelf(message));
}

pub fn Now() -> Jiffies {
    WithAccess(|access| access.Now())
}
