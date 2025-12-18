use simulator::{Message, ProcessId};

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct BCBMessageId {
    pub(super) process_id: ProcessId,
    pub(super) message_id: usize,
}

#[derive(Clone)]
pub enum BCBMessage<M: Message> {
    // Broadcast
    Initiate((BCBMessageId, M)),
    Signature(BCBMessageId),
    Certificate(usize, BCBMessageId),
    // Other
    Skip(M),
}

const ID_SIZE: usize = 128;
const SIG_SIZE: usize = 64;

impl<M: Message> Message for BCBMessage<M> {
    fn VirtualSize(&self) -> usize {
        match self {
            BCBMessage::Skip(m) => m.VirtualSize(),
            BCBMessage::Initiate((_, m)) => ID_SIZE + m.VirtualSize(),
            BCBMessage::Signature(_) => SIG_SIZE,
            BCBMessage::Certificate(quorum_size, _) => quorum_size * SIG_SIZE,
        }
    }
}
