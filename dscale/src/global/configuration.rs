use crate::{ProcessId, global::anykv, random::Seed, rank};

pub(crate) fn setup_global_configuration(proc_num: usize) {
    anykv::set::<usize>("proc_num", proc_num)
}

pub(crate) fn setup_local_configuration(id: ProcessId, base_seed: Seed) {
    // Prevent resonance between procs by changing seed a little bit
    anykv::set::<u64>(&format!("seeds/{}", id), base_seed + id as u64)
}

pub fn seed() -> Seed {
    anykv::get::<u64>(&format!("seeds/{}", rank()))
}

pub fn process_number() -> usize {
    anykv::get::<usize>("proc_num")
}
