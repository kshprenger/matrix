use std::{
    collections::{BTreeMap, btree_map::Keys},
    rc::Rc,
};

use log::debug;

use crate::{
    ProcessId, communication::DScaleMessage, global::set_process, process::MutableProcessHandle,
};

pub(crate) type HandlerMap = BTreeMap<ProcessId, MutableProcessHandle>; // btree for deterministic iterators

pub(crate) struct Nursery {
    procs: HandlerMap,
}

impl Nursery {
    pub(crate) fn new(procs: HandlerMap) -> Rc<Self> {
        Rc::new(Self { procs })
    }

    pub(crate) fn start_single(&self, id: ProcessId) {
        set_process(id);
        debug!("Starting P{id}");
        self.procs
            .get(&id)
            .expect("Invalid ProcessId")
            .borrow_mut()
            .start();
    }

    pub(crate) fn deliver(&self, from: ProcessId, to: ProcessId, m: DScaleMessage) {
        let mut handle = self.procs.get(&to).expect("Invalid ProcessId").borrow_mut();
        set_process(to);
        debug!("Executing step for From: P{} | To: P{}", to, from);
        match m {
            DScaleMessage::NetworkMessage(ptr) => handle.on_message(from, ptr),
            DScaleMessage::Timer(id) => handle.on_timer(id),
        }
    }

    pub(crate) fn keys(&self) -> Keys<'_, ProcessId, MutableProcessHandle> {
        self.procs.keys()
    }

    pub(crate) fn size(&self) -> usize {
        self.procs.len()
    }
}
