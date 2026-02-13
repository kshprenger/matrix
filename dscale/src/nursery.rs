use std::{
    collections::{BTreeMap, btree_map::Keys},
    rc::Rc,
};

use log::debug;

use crate::{
    ProcessId, communication::DScaleMessage, global::SetProcess, process::MutableProcessHandle,
};

pub(crate) type HandlerMap = BTreeMap<ProcessId, MutableProcessHandle>; // btree for deterministic iterators

pub(crate) struct Nursery {
    procs: HandlerMap,
}

impl Nursery {
    pub(crate) fn New(procs: HandlerMap) -> Rc<Self> {
        Rc::new(Self { procs })
    }

    pub(crate) fn StartSingle(&self, id: ProcessId) {
        SetProcess(id);
        debug!("Starting P{id}");
        self.procs
            .get(&id)
            .expect("Invalid ProcessId")
            .borrow_mut()
            .Start();
    }

    pub(crate) fn Deliver(&self, from: ProcessId, to: ProcessId, m: DScaleMessage) {
        let mut handle = self.procs.get(&to).expect("Invalid ProcessId").borrow_mut();
        SetProcess(to);
        debug!("Executing step for From: P{} | To: P{}", to, from);
        match m {
            DScaleMessage::NetworkMessage(ptr) => handle.OnMessage(from, ptr),
            DScaleMessage::Timer(id) => handle.OnTimer(id),
        }
    }

    pub(crate) fn Keys(&self) -> Keys<'_, ProcessId, MutableProcessHandle> {
        self.procs.keys()
    }

    pub(crate) fn Size(&self) -> usize {
        self.procs.len()
    }
}
