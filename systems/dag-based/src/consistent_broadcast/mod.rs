mod message;
pub(crate) use message::BCBMessage;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use matrix::{Broadcast, CurrentId, Message, MessagePtr, ProcessId, SendTo};

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
    fn AdversaryThreshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn QuorumSize(&self) -> usize {
        2 * self.AdversaryThreshold() + 1
    }

    fn NextUniqueMessageId(&mut self) -> BCBMessageId {
        self.message_id += 1;
        BCBMessageId {
            process_id: self.process_id,
            message_id: self.message_id,
        }
    }
}

impl ByzantineConsistentBroadcast {
    pub(crate) fn ReliablyBroadcast(&mut self, message: impl Message + 'static) {
        let next_id = self.NextUniqueMessageId();
        let shared = Rc::new(message);
        self.messages.insert(next_id, (shared.clone(), 0));
        Broadcast(BCBMessage::Initiate((next_id, shared)));
    }

    pub(crate) fn Start(&mut self, proc_num: usize) {
        self.process_id = CurrentId();
        self.proc_num = proc_num;
    }

    pub(crate) fn Process(
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
                    Some((message, _)) => return Some(MessagePtr::New(message)),
                }
            }
            BCBMessage::Initiate((id, m)) => {
                if id.process_id != self.process_id {
                    if self.waiting_certificates.contains(&id) {
                        self.waiting_certificates.remove(&id);
                        return Some(MessagePtr::New(m.clone()));
                    }
                    self.messages.insert(*id, (m.clone(), 0));
                }
                SendTo(from, BCBMessage::Signature(*id));
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
                        if message_state.1 == self.QuorumSize() {
                            Broadcast(BCBMessage::Certificate(self.QuorumSize(), *id));
                        }
                        return None;
                    }
                }
            }
        }
    }
}
