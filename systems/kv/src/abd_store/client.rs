use matrix::{
    global::{anykv, configuration},
    lincheck::history::{Entry, ExecutionHistory},
    *,
};

use rand::{Rng, SeedableRng, rngs::StdRng, seq::IndexedRandom};

use crate::abd_store::types::{Key, Value};

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
    current_op: Option<Entry>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            rng: None,
            keypool: vec![1, 3, 4, 6, 10],
            current_op: None,
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
        self.current_op.as_mut().map(|op| op.client = CurrentId());
        self.current_op.as_mut().map(|op| op.end = Now());
        match *response {
            ClientResponse::GetResponse(value) => {
                Debug!("Got get response from {from}. Value: {value}");
                self.current_op
                    .as_mut()
                    .map(|op| op.result = Some(value.to_string()));
            }
            ClientResponse::PutAck => {
                Debug!("Got PutAck from {from}");
                self.current_op.as_mut().map(|op| op.result = None);
            }
        }

        anykv::Modify::<ExecutionHistory>("linearizable_history", |h| {
            h.push(self.current_op.clone().unwrap());
        });

        ScheduleTimerAfter(Jiffies(100));
    }

    fn OnTimer(&mut self, _id: matrix::TimerId) {
        self.DoRandomOperation();
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

        self.current_op.as_mut().map(|op| op.start = Now());

        if random_bool {
            Debug!("Choosed operation: Get({random_key})");
            self.current_op
                .as_mut()
                .map(|op| op.operation = String::from(format!("Get({random_key})")));
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
