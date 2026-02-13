use std::{any::Any, cmp::Reverse, collections::BinaryHeap, rc::Rc};

use crate::{process::ProcessId, time::Jiffies};

pub trait Message: Any {
    // In bytes
    fn VirtualSize(&self) -> usize {
        usize::default()
    }
}

pub struct MessagePtr(pub Rc<dyn Message>);

impl MessagePtr {
    pub fn TryAs<T: 'static>(&self) -> Option<Rc<T>> {
        match (self.0.clone() as Rc<dyn Any>).downcast::<T>() {
            Err(_) => None,
            Ok(m) => Some(m),
        }
    }

    pub fn Is<T: 'static>(&self) -> bool {
        (self.0.clone() as Rc<dyn Any>).is::<T>()
    }

    pub fn As<T: 'static>(self) -> Rc<T> {
        (self.0 as Rc<dyn Any>).downcast::<T>().unwrap()
    }
}

#[derive(Clone)]
pub struct ProcessStep {
    pub(crate) source: ProcessId,
    pub(crate) dest: ProcessId,
    pub(crate) message: Rc<dyn Message>,
}

#[derive(Clone)]
pub struct RoutedMessage {
    pub(crate) arrival_time: Jiffies,
    pub(crate) step: ProcessStep,
}

impl PartialEq for RoutedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.arrival_time.eq(&other.arrival_time)
    }
}

impl Eq for RoutedMessage {}

impl PartialOrd for RoutedMessage {
    fn ge(&self, other: &Self) -> bool {
        self.arrival_time.ge(&other.arrival_time)
    }
    fn le(&self, other: &Self) -> bool {
        self.arrival_time.le(&other.arrival_time)
    }
    fn gt(&self, other: &Self) -> bool {
        self.arrival_time.gt(&other.arrival_time)
    }
    fn lt(&self, other: &Self) -> bool {
        self.arrival_time.lt(&other.arrival_time)
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.arrival_time.partial_cmp(&other.arrival_time)
    }
}

impl Ord for RoutedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.arrival_time.cmp(&other.arrival_time)
    }
}

pub type TimePriorityMessageQueue = BinaryHeap<Reverse<RoutedMessage>>;
