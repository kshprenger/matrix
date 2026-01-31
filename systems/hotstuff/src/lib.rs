// https://arxiv.org/pdf/1803.05069

use std::{collections::HashMap, rc::Rc};

use matrix::{
    Broadcast, Message, MessagePtr, ProcessHandle, ProcessId, Rank, SendTo,
    global::configuration::ProcessNumber,
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
    fn Start(&mut self) {
        if Rank() == self.GetLeader() {
            Broadcast(HSMessage::Propose(self.CreateLeaf()));
        }
    }
    fn OnMessage(&mut self, from: ProcessId, message: MessagePtr) {
        match message.As::<HSMessage>().as_ref() {
            HSMessage::Propose(b_new) => {
                if b_new.height > self.vheight
                    && (self.Extends(b_new) || b_new.height > self.b_lock.as_ref().unwrap().height)
                {
                    self.vheight = b_new.height;

                    SendTo(self.GetLeader(), HSMessage::Vote);
                } else {
                    // Store in prio queue buffer
                    todo!()
                }
            }
            HSMessage::Vote => {}
        }
    }
    fn OnTimer(&mut self, _id: matrix::TimerId) {
        unreachable!()
    }
}

// Utils
impl ChainedHotstuff {
    fn CreateLeaf(&self) -> Rc<Node> {
        // Infinite source of txns
        let parent = self.leaf.clone();
        Rc::new(Node {
            parent: parent.clone(),
            height: parent.height + 1,
        })
    }

    fn GetLeader(&self) -> ProcessId {
        (self.vheight % ProcessNumber()) + 1
    }

    fn Extends(&self, child: &Rc<Node>) -> bool {
        if self.b_lock.is_none() {
            // Genesis case
            return true;
        } else {
            Rc::ptr_eq(&child.parent, self.b_lock.as_ref().unwrap())
        }
    }
}
