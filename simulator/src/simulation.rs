use std::process::exit;

use log::{error, info};

use crate::{
    network::{BandwidthType, Network},
    process::{ProcessHandle, ProcessId},
    progress::Bar,
    random::{self},
    time::{Jiffies, Now},
};

pub struct Simulation<P>
where
    P: ProcessHandle,
{
    network: Network<P>,
    max_time: Jiffies,
    progress_bar: Bar,
}

const K_PROGRESS_TIMES: usize = 10;

impl<P> Simulation<P>
where
    P: ProcessHandle,
{
    pub(crate) fn New(
        seed: random::Seed,
        max_time: Jiffies,
        max_network_latency: Jiffies,
        bandwidth_type: BandwidthType,
        procs: Vec<(ProcessId, P)>,
    ) -> Self {
        let _ = env_logger::try_init();

        Self {
            network: Network::New(
                seed,
                max_network_latency,
                bandwidth_type,
                procs.into_iter().collect(),
            ),
            max_time: max_time,
            progress_bar: Bar::New(max_time, max_time.0 / K_PROGRESS_TIMES),
        }
    }

    pub fn Run(&mut self) {
        self.network.Start();

        while self.KeepRunning() {
            if !self.Step() {
                error!("DEADLOCK! (ﾉಥ益ಥ）ﾉ ┻━┻ Try with RUST_LOG=debug");
                exit(1)
            }
            self.progress_bar.MakeProgress(Now());
        }

        info!("Looks good! ヽ(‘ー`)ノ");
    }
}

impl<P> Simulation<P>
where
    P: ProcessHandle,
{
    fn KeepRunning(&mut self) -> bool {
        Now() < self.max_time
    }

    fn Step(&mut self) -> bool {
        self.network.Step()
    }
}
