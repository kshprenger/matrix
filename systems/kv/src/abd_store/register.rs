// https://dl.acm.org/doi/pdf/10.1145/200836.200869

use std::collections::HashMap;

use matrix::*;

use crate::abd_store::{
    client::ClientResponse,
    types::{ClientId, Key, REPLICA_POOL_NAME, ReadSequence, Timestamp, Value},
};

pub(crate) struct RoutedRegisterOp {
    pub(crate) key: Key,
    pub(crate) op: RegisterOps,
}

pub(crate) enum RegisterOps {
    RegisterReadRequest(ReadSequence),
    RegisterReadResponse(Value, Timestamp, ReadSequence),
    RegisterWriteRequest(Value, Timestamp),
    RegisterWriteAck(Value, Timestamp),
}

impl Message for RoutedRegisterOp {}

// Manual coroutines
enum CoroResumeAfterReadQuorum {
    Write(ClientId, Value),
    Read(ClientId),
}

// Manual coroutines
enum CoroResumeAfterWriteQuorum {
    Write(ClientId),
    Read(ClientId, Value),
}

struct PendingReadQuorum {
    resume: CoroResumeAfterReadQuorum,
    read_quorum: Vec<(Value, Timestamp, ReadSequence)>,
}

struct PendingWriteQuorum {
    resume: CoroResumeAfterWriteQuorum,
    write_quorum: Vec<(Value, Timestamp)>,
}

pub(crate) struct MWMRAtomicRegister {
    key: Key,
    local_value: Value,
    local_ts: usize,
    t: usize,
    r: usize,
    pending_read_quorums: HashMap<ReadSequence, PendingReadQuorum>,
    pending_write_quorums: HashMap<Timestamp, PendingWriteQuorum>,
}

impl MWMRAtomicRegister {
    pub(crate) fn New(key: Key) -> Self {
        Self {
            key,
            local_value: 0,
            local_ts: 0,
            t: 0,
            r: 0,
            pending_read_quorums: HashMap::new(),
            pending_write_quorums: HashMap::new(),
        }
    }

    pub(crate) fn Write(&mut self, client: ClientId, value: Value) {
        self.r += 1;
        Debug!("[r == {}] Gathering read quorum for Write...", self.r);
        self.pending_read_quorums.insert(
            self.r,
            PendingReadQuorum {
                resume: CoroResumeAfterReadQuorum::Write(client, value),
                read_quorum: Vec::new(),
            },
        );
        BroadcastWithinPool(
            REPLICA_POOL_NAME,
            RoutedRegisterOp {
                key: self.key,
                op: RegisterOps::RegisterReadRequest(self.r),
            },
        );
        return;
    }

    pub(crate) fn Read(&mut self, client: ClientId) {
        self.r += 1;
        Debug!("[r == {}]. Gathering read quorum for Read...", self.r);
        self.pending_read_quorums.insert(
            self.r,
            PendingReadQuorum {
                resume: CoroResumeAfterReadQuorum::Read(client),
                read_quorum: Vec::new(),
            },
        );
        BroadcastWithinPool(
            REPLICA_POOL_NAME,
            RoutedRegisterOp {
                key: self.key,
                op: RegisterOps::RegisterReadRequest(self.r),
            },
        );
    }

    pub(crate) fn Serve(
        &mut self,
        op: &RegisterOps,
        from: ProcessId,
        key: Key,
        quorum_size: usize,
    ) {
        match *op {
            RegisterOps::RegisterReadRequest(r_) => {
                SendTo(
                    from,
                    RoutedRegisterOp {
                        key,
                        op: RegisterOps::RegisterReadResponse(self.local_value, self.local_ts, r_),
                    },
                );
                return;
            }

            RegisterOps::RegisterWriteRequest(v_, t_) => {
                if t_ > self.local_ts || (t_ == self.local_ts && v_ > self.local_value) {
                    self.local_value = v_;
                    self.local_ts = t_;
                }
                SendTo(
                    from,
                    RoutedRegisterOp {
                        key,
                        op: RegisterOps::RegisterWriteAck(v_, t_),
                    },
                );
                return;
            }

            RegisterOps::RegisterReadResponse(v_, t_, r) => {
                let qourum_info = self.pending_read_quorums.get_mut(&r).unwrap();
                qourum_info.read_quorum.push((v_, t_, r));

                if qourum_info.read_quorum.len() == quorum_size {
                    match qourum_info.resume {
                        CoroResumeAfterReadQuorum::Write(client, saved_value) => {
                            Debug!("Gathered read quorum for Write");
                            Debug!("Resuming Write...");
                            let t_ = qourum_info
                                .read_quorum
                                .iter()
                                .map(|(_, t, _)| t)
                                .max()
                                .expect("Should not be empty");
                            self.t = t_ + 1;

                            self.pending_write_quorums.insert(
                                self.t,
                                PendingWriteQuorum {
                                    resume: CoroResumeAfterWriteQuorum::Write(client),
                                    write_quorum: Vec::new(),
                                },
                            );

                            Debug!("Gathering write quorum for Write...");
                            BroadcastWithinPool(
                                REPLICA_POOL_NAME,
                                RoutedRegisterOp {
                                    key,
                                    op: RegisterOps::RegisterWriteRequest(saved_value, self.t),
                                },
                            );
                        }
                        CoroResumeAfterReadQuorum::Read(client) => {
                            Debug!("Gathered read quorum for Read");
                            Debug!("Resuming Read...");
                            // let v_m be the largest value with the highest timestamp t_m
                            let (v_m, t_m, _) = qourum_info
                                .read_quorum
                                .iter()
                                .max_by(|l, r| ((l.1, l.0)).cmp(&(r.1, r.0)))
                                .copied()
                                .unwrap();

                            self.pending_write_quorums.insert(
                                t_m,
                                PendingWriteQuorum {
                                    resume: CoroResumeAfterWriteQuorum::Read(client, v_m),
                                    write_quorum: Vec::new(),
                                },
                            );

                            Debug!("Gathering write quorum for Read...");
                            BroadcastWithinPool(
                                REPLICA_POOL_NAME,
                                RoutedRegisterOp {
                                    key,
                                    op: RegisterOps::RegisterWriteRequest(v_m, t_m),
                                },
                            );
                        }
                    }
                }
            }

            RegisterOps::RegisterWriteAck(v, t) => {
                let qourum_info = self.pending_write_quorums.get_mut(&t).unwrap();
                qourum_info.write_quorum.push((v, t));

                if qourum_info.write_quorum.len() == quorum_size {
                    match qourum_info.resume {
                        CoroResumeAfterWriteQuorum::Write(client) => {
                            Debug!("Gathered write quorum for Write");
                            Debug!("Resuming Write...");
                            SendTo(client, ClientResponse::PutAck);
                        }
                        CoroResumeAfterWriteQuorum::Read(client, saved_value) => {
                            Debug!("Gathered write quorum for Read");
                            Debug!("Resuming Read...");
                            SendTo(client, ClientResponse::GetResponse(saved_value));
                        }
                    }
                }
            }
        }
    }
}
