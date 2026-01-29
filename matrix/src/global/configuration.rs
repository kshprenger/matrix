use crate::{ProcessId, Rank, global::anykv, random::Seed};

pub(crate) fn SetupGlobalConfiguration(proc_num: usize) {
    anykv::Set::<usize>("proc_num", proc_num)
}

pub(crate) fn SetupLocalConfiguration(id: ProcessId, base_seed: Seed) {
    // Prevent resonance between procs by changing seed a little bit
    anykv::Set::<u64>(&format!("seeds/{}", id), base_seed + id as u64)
}

pub fn Seed() -> Seed {
    anykv::Get::<u64>(&format!("seeds/{}", Rank()))
}

pub fn ProcessNumber() -> usize {
    anykv::Get::<usize>("proc_num")
}
