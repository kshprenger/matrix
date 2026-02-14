use dscale::{
    global::{anykv, configuration},
    *,
};

use rand::{Rng, SeedableRng, rngs::StdRng, seq::IndexedRandom};

use crate::abd_store::types::{Key, Value};

#[derive(Default, Clone)]
pub struct ExecutionHistoryEntry {
    pub client: ProcessId,
    pub operation: String,
    pub result: Option<Value>,
    pub start: Jiffies,
    pub end: Jiffies,
}
pub type ExecutionHistory = Vec<ExecutionHistoryEntry>;

pub(crate) enum ClientReq {
    PutRequest(Key, Value),
    GetRequest(Key),
}

pub(crate) enum ClientResponse {
    GetResponse(Value),
    PutAck,
}

impl Message for ClientReq {}
impl Message for ClientResponse {}

pub struct Client {
    rng: Option<StdRng>,
    keypool: Vec<Key>,
    current_op: ExecutionHistoryEntry,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            rng: None,
            keypool: vec![1, 3, 4, 6, 10],
            current_op: ExecutionHistoryEntry::default(),
        }
    }
}

impl ProcessHandle for Client {
    fn start(&mut self) {
        self.rng = Some(StdRng::seed_from_u64(configuration::seed()));
        schedule_timer_after(Jiffies(100));
    }

    fn on_message(&mut self, from: dscale::ProcessId, message: dscale::MessagePtr) {
        let response = message.as_type::<ClientResponse>();
        self.current_op.client = rank();
        self.current_op.end = now();
        match *response {
            ClientResponse::GetResponse(value) => {
                debug_process!("Got get response from {from}. Value: {value}");
                self.current_op.result = Some(value);
            }
            ClientResponse::PutAck => {
                debug_process!("Got PutAck from {from}");
                self.current_op.result = None;
            }
        }

        anykv::modify::<ExecutionHistory>("linearizable_history", |h| {
            h.push(self.current_op.clone());
        });

        schedule_timer_after(Jiffies(100));
    }

    fn on_timer(&mut self, _id: dscale::TimerId) {
        self.do_random_operation();
    }
}

impl Client {
    fn choose_key(&mut self) -> Key {
        self.keypool
            .choose(self.rng.as_mut().unwrap())
            .copied()
            .unwrap()
    }

    fn choose_value(&self) -> Value {
        global_unique_id() // Make values monotonous
    }

    fn choose_operation(&mut self) -> ClientReq {
        let random_bool = self.rng.as_mut().unwrap().random::<bool>();
        let random_key = self.choose_key();

        self.current_op.start = now();

        if random_bool {
            debug_process!("Choosed operation: Get({random_key})");
            self.current_op.operation = String::from(format!("Get({random_key})"));
            ClientReq::GetRequest(random_key)
        } else {
            let value = self.choose_value();
            debug_process!("Choosed operation: Put({random_key},{value})");
            self.current_op.operation = String::from(format!("Put({random_key},{value})"));
            ClientReq::PutRequest(random_key, value)
        }
    }

    fn do_random_operation(&mut self) {
        let target = choose_from_pool("Replicas");
        let operation = self.choose_operation();
        send_to(target, operation);
        debug_process!("Sent operation to {target}");
    }
}
