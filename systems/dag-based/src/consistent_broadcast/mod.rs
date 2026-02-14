mod message;
pub(crate) use message::BCBMessage;
pub(crate) use message::ID_SIZE;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dscale::{Message, MessagePtr, ProcessId, broadcast, rank, send_to};

use crate::consistent_broadcast::message::BCBMessageId;

// Introduction to Reliable and Secure Distributed Programming
// Algorithm 3.17: Signed Echo Broadcast
#[derive(Default)]
pub struct ByzantineConsistentBroadcast {
    messages: HashMap<BCBMessageId, (Rc<dyn Message>, usize)>, // usize -> signature count, once it reaches 2f+1 message pops out
    waiting_certificates: HashSet<BCBMessageId>,
    process_id: ProcessId,
    message_id: usize,
    proc_num: usize,
}

impl ByzantineConsistentBroadcast {
    fn adversary_threshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn quorum_size(&self) -> usize {
        2 * self.adversary_threshold() + 1
    }

    fn next_unique_message_id(&mut self) -> BCBMessageId {
        self.message_id += 1;
        BCBMessageId {
            process_id: self.process_id,
            message_id: self.message_id,
        }
    }
}

impl ByzantineConsistentBroadcast {
    pub(crate) fn reliably_broadcast(&mut self, message: impl Message + 'static) {
        let next_id = self.next_unique_message_id();
        let shared = Rc::new(message);
        self.messages.insert(next_id, (shared.clone(), 0));
        broadcast(BCBMessage::Initiate((next_id, shared)));
    }

    pub(crate) fn start(&mut self, proc_num: usize) {
        self.process_id = rank();
        self.proc_num = proc_num;
    }

    pub(crate) fn process(
        &mut self,
        from: ProcessId,
        message: Rc<BCBMessage>,
    ) -> Option<MessagePtr> {
        match message.as_ref() {
            BCBMessage::Certificate(_, id) => {
                match self.messages.remove(&id) {
                    // Due to network latency we got certificate gathered by some other quorum (not including us)
                    None => {
                        self.waiting_certificates.insert(*id);
                        return None;
                    }
                    Some((message, _)) => return Some(MessagePtr(message)),
                }
            }
            BCBMessage::Initiate((id, m)) => {
                if id.process_id != self.process_id {
                    if self.waiting_certificates.contains(&id) {
                        self.waiting_certificates.remove(&id);
                        return Some(MessagePtr(m.clone()));
                    }
                    self.messages.insert(*id, (m.clone(), 0));
                }
                send_to(from, BCBMessage::Signature(*id));
                return None;
            }
            BCBMessage::Signature(id) => {
                match self.messages.get_mut(&id) {
                    None => {
                        // Message already gathered quorum and was poped.
                        return None;
                    }
                    Some(message_state) => {
                        message_state.1 += 1;
                        if message_state.1 == self.quorum_size() {
                            broadcast(BCBMessage::Certificate(self.proc_num, *id));
                        }
                        return None;
                    }
                }
            }
        }
    }
}
