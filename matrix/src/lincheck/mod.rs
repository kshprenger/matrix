// https://www.cs.ox.ac.uk/people/gavin.lowe/LinearizabiltyTesting/paper.pdf
pub mod history;

use std::collections::HashMap;

use crate::{
    Jiffies,
    lincheck::history::{Entry, ExecutionHistory, Key, SingleKeyOperation, Value},
};

// Wing-Gong like checker
// TODO: JIT version
pub fn CheckLinearizable(history: &ExecutionHistory) -> bool {
    let mut keys_history: HashMap<Key, Vec<Entry>> = HashMap::new();
    let mut max_time = 0;

    for Entry in history.iter().cloned() {
        max_time = max_time.max(Entry.end.0);
        keys_history
            .entry(Entry.key.clone())
            .or_default()
            .push(Entry);
    }

    if keys_history.is_empty() {
        return true;
    }

    for (key, mut ops) in keys_history {
        ops.sort_by_key(|op| op.end);

        if !CheckSingleKey(&ops) {
            println!("Linearizability violation for key {}!", key);
            return false;
        }
    }

    println!("LinChecker: History is linearizable!");
    true
}

fn CheckSingleKey(entries: &[Entry]) -> bool {
    let mut used = vec![false; entries.len()];
    Search(entries, &mut used, 0, Value::default())
}

fn Search(entries: &[Entry], used: &mut [bool], count: usize, current_value: Value) -> bool {
    if count == entries.len() {
        return true;
    }

    let mut min_end = Jiffies(usize::MAX);
    for i in 0..entries.len() {
        if !used[i] && entries[i].end < min_end {
            min_end = entries[i].end;
        }
    }

    for i in 0..entries.len() {
        if used[i] {
            continue;
        }
        let Entry = &entries[i];

        if Entry.start > min_end {
            continue;
        }

        let consistent = match Entry.operation {
            SingleKeyOperation::Read(ref v) => *v == current_value.clone(),
            SingleKeyOperation::Write(_) => true,
        };

        if consistent {
            used[i] = true;
            let next_value = match Entry.operation {
                SingleKeyOperation::Read(_) => current_value.clone(),
                SingleKeyOperation::Write(ref v) => v.clone(),
            };

            if Search(entries, used, count + 1, next_value) {
                return true;
            }

            used[i] = false;
        }
    }

    false
}
