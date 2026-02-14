use std::{cell::RefCell, rc::Rc};

use crate::time::Jiffies;

pub(crate) type SharedActor = Rc<RefCell<dyn SimulationActor>>;

pub(crate) trait SimulationActor {
    fn start(&mut self);
    fn step(&mut self);
    fn peek_closest(&self) -> Option<Jiffies>;
}

pub(crate) trait EventSubmitter {
    type Event;
    fn submit(&mut self, events: &mut Vec<Self::Event>);
}
