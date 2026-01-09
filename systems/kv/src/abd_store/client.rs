use matrix::{global::configuration, *};

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

impl Message for ClientReq {
    fn VirtualSize(&self) -> usize {
        usize::default()
    }
}

impl Message for ClientResponse {
    fn VirtualSize(&self) -> usize {
        usize::default()
    }
}

pub struct Client {
    rng: Option<StdRng>,
    keypool: Vec<Key>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            rng: None,
            keypool: vec![1],
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
        match *response {
            ClientResponse::GetResponse(value) => {
                Debug!("Got get response from {from}. Value: {value}")
            }
            ClientResponse::PutAck => {
                Debug!("Got PutAck from {from}")
            }
        }
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

        if random_bool {
            Debug!("Choosed operation: Get({random_key})");
            ClientReq::GetRequest(random_key)
        } else {
            let value = self.ChooseValue();
            Debug!("Choosed operation: Put({random_key},{value})");
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
