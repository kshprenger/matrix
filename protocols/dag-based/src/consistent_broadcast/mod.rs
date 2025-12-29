mod message;
pub(crate) use message::BCBMessage;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use simulator::{Broadcast, Configuration, CurrentId, Message, MessagePtr, ProcessId, SendTo};

use crate::consistent_broadcast::message::BCBMessageId;

// Introduction to Reliable and Secure Distributed Programming
// Algorithm 3.17: Signed Echo Broadcast
//
// Works as wrapper aroung generic dag-based bft consensus.
// So it acts like process handle too.
pub struct ByzantineConsistentBroadcast {
    messages: HashMap<BCBMessageId, (Rc<dyn Message>, usize)>, // usize -> signature count, once it reaches 2f+1 message pops out
    waiting_certificates: HashSet<BCBMessageId>,
    process_id: ProcessId,
    message_id: usize,
    proc_num: usize,
}

impl ByzantineConsistentBroadcast {
    pub fn New() -> Self {
        Self {
            messages: HashMap::new(),
            waiting_certificates: HashSet::new(),
            process_id: 0,
            message_id: 0,
            proc_num: 0,
        }
    }
}

impl ByzantineConsistentBroadcast {
    fn AdversaryThreshold(&self) -> usize {
        (self.proc_num - 1) / 3
    }

    fn QuorumSize(&self) -> usize {
        2 * self.AdversaryThreshold() - 1
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

    pub(crate) fn Bootstrap(&mut self, configuration: Configuration) {
        self.process_id = CurrentId();
        self.proc_num = configuration.proc_num;
    }

    pub(crate) fn Process(
        &mut self,
        from: ProcessId,
        message: Rc<BCBMessage>,
    ) -> Option<MessagePtr> {
        match message.as_ref() {
            BCBMessage::Certificate(_, id) => {
                match self.messages.remove(&id) {
                    // Due to network latency we got certificate gathered by some quorum
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
                        if message_state.1 >= self.QuorumSize() {
                            Broadcast(BCBMessage::Certificate(self.QuorumSize(), *id));
                        }
                        return None;
                    }
                }
            }
        }
    }
}
