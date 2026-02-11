use std::rc::Rc;

use dscale::{Message, ProcessId};

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct BCBMessageId {
    pub(super) process_id: ProcessId,
    pub(super) message_id: usize,
}

pub enum BCBMessage {
    Initiate((BCBMessageId, Rc<dyn Message>)),
    Signature(BCBMessageId),
    Certificate(usize, BCBMessageId),
}

const ID_SIZE: usize = 128;
const SIG_SIZE: usize = 64; // For example Ed25519 or Secp256k1

impl Message for BCBMessage {
    fn VirtualSize(&self) -> usize {
        match self {
            BCBMessage::Initiate((_, m)) => ID_SIZE + m.VirtualSize(),
            BCBMessage::Signature(_) => SIG_SIZE,
            BCBMessage::Certificate(quorum_size, _) => quorum_size * SIG_SIZE,
        }
    }
}
