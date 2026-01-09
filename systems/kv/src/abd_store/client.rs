use matrix::{
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
    fn Bootstrap(&mut self) {
        self.rng = Some(StdRng::seed_from_u64(configuration::Seed()));
        ScheduleTimerAfter(Jiffies(100));
    }

    fn OnMessage(&mut self, from: matrix::ProcessId, message: matrix::MessagePtr) {
        let response = message.As::<ClientResponse>();
        self.current_op.client = CurrentId();
        self.current_op.end = Now();
        match *response {
            ClientResponse::GetResponse(value) => {
                Debug!("Got get response from {from}. Value: {value}");
                self.current_op.result = Some(value);
            }
            ClientResponse::PutAck => {
                Debug!("Got PutAck from {from}");
                self.current_op.result = None;
            }
        }

        anykv::Modify::<ExecutionHistory>("linearizable_history", |h| {
            h.push(self.current_op.clone());
        });
    }

    fn OnTimer(&mut self, _id: matrix::TimerId) {
        self.DoRandomOperation();
        ScheduleTimerAfter(Jiffies(100));
    }
}

impl Client {
    fn ChooseServer(&mut self) -> ProcessId {
        ListPool("Replicas")
            .choose(self.rng.as_mut().unwrap())
            .copied()
            .unwrap()
    }

    fn ChooseKey(&mut self) -> Key {
        self.keypool
            .choose(self.rng.as_mut().unwrap())
            .copied()
            .unwrap()
    }

    fn ChooseValue(&self) -> Value {
        GlobalUniqueId() // Make values monotonous
    }

    fn ChooseOperation(&mut self) -> ClientReq {
        let random_bool = self.rng.as_mut().unwrap().random::<bool>();
        let random_key = self.ChooseKey();

        self.current_op.start = Now();

        if random_bool {
            Debug!("Choosed operation: Get({random_key})");
            self.current_op.operation = String::from(format!("Get({random_key})"));
            ClientReq::GetRequest(random_key)
        } else {
            let value = self.ChooseValue();
            Debug!("Choosed operation: Put({random_key},{value})");
            self.current_op.operation = String::from(format!("Put({random_key},{value})"));
            ClientReq::PutRequest(random_key, value)
        }
    }

    fn DoRandomOperation(&mut self) {
        let target = self.ChooseServer();
        let operation = self.ChooseOperation();
        SendTo(target, operation);
        Debug!("Sent operation to {target}");
    }
}
