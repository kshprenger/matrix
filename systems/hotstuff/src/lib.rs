// https://arxiv.org/pdf/1803.05069

use std::{collections::HashMap, rc::Rc};

use dscale::{
    Message, MessagePtr, ProcessHandle, ProcessId, broadcast,
    global::configuration::process_number, rank, send_to,
};

type QCId = usize;

pub struct Node {
    parent: Rc<Node>,
    height: usize,
}

pub enum HSMessage {
    Propose(Rc<Node>),
    Vote,
}

impl Message for HSMessage {}

pub struct ChainedHotstuff {
    vheight: usize,
    leaf: Rc<Node>,
    b_lock: Option<Rc<Node>>,
    pending_quorums: HashMap<QCId, usize>,
}

impl ProcessHandle for ChainedHotstuff {
    fn start(&mut self) {
        if rank() == self.get_leader() {
            broadcast(HSMessage::Propose(self.create_leaf()));
        }
    }
    fn on_message(&mut self, from: ProcessId, message: MessagePtr) {
        match message.as_type::<HSMessage>().as_ref() {
            HSMessage::Propose(b_new) => {
                if b_new.height > self.vheight
                    && (self.extends(b_new) || b_new.height > self.b_lock.as_ref().unwrap().height)
                {
                    self.vheight = b_new.height;

                    send_to(self.get_leader(), HSMessage::Vote);
                } else {
                    // Store in prio queue buffer
                    todo!()
                }
            }
            HSMessage::Vote => {}
        }
    }
    fn on_timer(&mut self, _id: dscale::TimerId) {
        unreachable!()
    }
}

// Utils
impl ChainedHotstuff {
    fn create_leaf(&self) -> Rc<Node> {
        // Infinite source of txns
        let parent = self.leaf.clone();
        Rc::new(Node {
            parent: parent.clone(),
            height: parent.height + 1,
        })
    }

    fn get_leader(&self) -> ProcessId {
        (self.vheight % process_number()) + 1
    }

    fn extends(&self, child: &Rc<Node>) -> bool {
        if self.b_lock.is_none() {
            // Genesis case
            return true;
        } else {
            Rc::ptr_eq(&child.parent, self.b_lock.as_ref().unwrap())
        }
    }
}
