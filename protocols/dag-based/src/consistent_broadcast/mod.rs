mod access;
mod message;

use std::collections::{HashMap, HashSet};

use simulator::{Access, Message, ProcessHandle, ProcessId};

use crate::consistent_broadcast::{
    access::BCBAccess,
    message::{BCBMessage, BCBMessageId},
};

// Introduction to Reliable and Secure Distributed Programming
// Algorithm 3.17: Signed Echo Broadcast
//
// Works as wrapper aroung generic dag-based bft consensus.
// So it acts like process handle too.
pub struct ByzantineConsistentBroadcast<M, H>
where
    M: Message,
    H: ProcessHandle<M>,
{
    dag_based_consensus: H,
    messages: HashMap<BCBMessageId, (M, usize)>, // usize -> signature count, once it reaches 2f+1 message pops out
    waiting_certificates: HashSet<BCBMessageId>,
    process_id: ProcessId,
    message_id: usize,
    proc_num: usize,
}

impl<M, H> ByzantineConsistentBroadcast<M, H>
where
    M: Message,
    H: ProcessHandle<M>,
{
    pub fn Wrap(consensus: H) -> Self {
        Self {
            dag_based_consensus: consensus,
            messages: HashMap::new(),
            waiting_certificates: HashSet::new(),
            process_id: 0,
            message_id: 0,
            proc_num: 0,
        }
    }
}

impl<M, H> ByzantineConsistentBroadcast<M, H>
where
    M: Message,
    H: ProcessHandle<M>,
{
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

    fn InitiateBroadcasts(&mut self, messages: Vec<M>, access: &mut impl Access<BCBMessage<M>>) {
        messages.into_iter().for_each(|message| {
            let next_id = self.NextUniqueMessageId();
            self.messages.insert(next_id, (message.clone(), 0));
            access.Broadcast(BCBMessage::Initiate((next_id, message)));
        });
    }

    fn WithInnerAccess<F, A>(&mut self, f: F, outer_access: &mut A)
    where
        M: Message,
        A: Access<BCBMessage<M>>,
        F: FnOnce(&mut Self, &mut BCBAccess<'_, M, A>),
    {
        let mut bcb_access = BCBAccess::Wrap(outer_access);
        f(self, &mut bcb_access);
        self.InitiateBroadcasts(bcb_access.scheduled_broadcasts, outer_access)
    }
}

impl<M, H> ProcessHandle<BCBMessage<M>> for ByzantineConsistentBroadcast<M, H>
where
    M: Message,
    H: ProcessHandle<M>,
{
    fn Bootstrap(
        &mut self,
        configuration: simulator::Configuration,
        access: &mut impl Access<BCBMessage<M>>,
    ) {
        self.process_id = configuration.assigned_id;
        self.proc_num = configuration.proc_num;

        self.WithInnerAccess(
            |bcb, bcb_access| {
                bcb.dag_based_consensus.Bootstrap(configuration, bcb_access);
            },
            access,
        );
    }

    fn OnMessage(
        &mut self,
        from: ProcessId,
        message: BCBMessage<M>,
        access: &mut impl Access<BCBMessage<M>>,
    ) {
        match message {
            BCBMessage::Skip(m) => {
                self.WithInnerAccess(
                    |bcb, bcb_access| {
                        bcb.dag_based_consensus.OnMessage(from, m, bcb_access);
                    },
                    access,
                );
            }
            BCBMessage::Certificate(_, id) => {
                match self.messages.remove(&id) {
                    // Due to network latency we got certificate gathered by some quorum
                    None => {
                        self.waiting_certificates.insert(id);
                    }
                    Some((message, _)) => {
                        self.WithInnerAccess(
                            |bcb, bcb_access| {
                                bcb.dag_based_consensus.OnMessage(from, message, bcb_access);
                            },
                            access,
                        );
                    }
                }
            }
            BCBMessage::Initiate((id, m)) => {
                if id.process_id != self.process_id {
                    if self.waiting_certificates.contains(&id) {
                        self.WithInnerAccess(
                            |bcb, bcb_access| {
                                bcb.dag_based_consensus.OnMessage(from, m, bcb_access);
                            },
                            access,
                        );
                        return;
                    }
                    self.messages.insert(id, (m, 0));
                }
                access.SendTo(from, BCBMessage::Signature(id));
            }
            BCBMessage::Signature(id) => {
                match self.messages.get_mut(&id) {
                    None => {
                        // Message already gathered quorum and was poped.
                    }
                    Some(message_state) => {
                        message_state.1 += 1;
                        if message_state.1 >= self.QuorumSize() {
                            access.Broadcast(BCBMessage::Certificate(self.QuorumSize(), id));
                        }
                    }
                }
            }
        }
    }
}
