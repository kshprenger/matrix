pub mod client;
pub mod lin_checker;
pub mod register;
pub mod types;

use std::collections::HashMap;

use dscale::{global::configuration::process_number, *};

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
    fn quorum_size(&self) -> usize {
        self.proc_num / 2 + 1
    }

    fn find_register(&mut self, key: Key) -> &mut MWMRAtomicRegister {
        self.registers
            .entry(key)
            .or_insert(MWMRAtomicRegister::new(key))
    }
}

impl ProcessHandle for Replica {
    fn start(&mut self) {
        self.proc_num = process_number()
    }

    fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {
        if let Some(client_op) = message.try_as::<ClientReq>() {
            match *client_op {
                ClientReq::GetRequest(key) => {
                    debug_process!("Client {from} requested Get({key})");
                    self.find_register(key).read(from);
                }
                ClientReq::PutRequest(key, value) => {
                    debug_process!("Client {from} requested Put({key},{value})");
                    self.find_register(key).write(from, value);
                }
            }
            return;
        }

        let register_op = message.as_type::<RoutedRegisterOp>();
        let quorum_size = self.quorum_size();
        let register = self.find_register(register_op.key);
        register.serve(&register_op.op, from, register_op.key, quorum_size);
    }

    fn on_timer(&mut self, _id: TimerId) {}
}
