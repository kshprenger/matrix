pub mod client;
pub mod lin_checker;
pub mod register;
pub mod types;

use std::collections::HashMap;

use matrix::*;

use crate::abd_store::{
    client::ClientReq,
    register::{MWMRAtomicRegister, RoutedRegisterOp},
    types::Key,
};

#[derive(Default)]
pub struct Replica {
    proc_num: usize,
    registers: HashMap<Key, MWMRAtomicRegister>,
}

impl Replica {
    fn QuorumSize(&self) -> usize {
        self.proc_num / 2 + 1
    }

    fn FindRegister(&mut self, key: Key) -> &mut MWMRAtomicRegister {
        self.registers
            .entry(key)
            .or_insert(MWMRAtomicRegister::New(key))
    }
}

impl ProcessHandle for Replica {
    fn Bootstrap(&mut self) {
        // Do nothing
    }

    fn OnMessage(&mut self, from: matrix::ProcessId, message: matrix::MessagePtr) {
        if let Some(client_op) = message.TryAs::<ClientReq>() {
            match *client_op {
                ClientReq::GetRequest(key) => {
                    Debug!("Client {from} requested Get({key})");
                    self.FindRegister(key).Read(from);
                }
                ClientReq::PutRequest(key, value) => {
                    Debug!("Client {from} requested Put({key},{value})");
                    self.FindRegister(key).Write(from, value);
                }
            }
            return;
        }

        let register_op = message.As::<RoutedRegisterOp>();
        let quorum_size = self.QuorumSize();
        let register = self.FindRegister(register_op.key);
        register.Serve(&register_op.op, from, register_op.key, quorum_size);
    }
}
