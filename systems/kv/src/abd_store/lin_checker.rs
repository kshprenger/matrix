use crate::abd_store::client::ExecutionHistory;
use crate::abd_store::types::{Key, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Read(Value),
    Write(Value),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub key: Key,
    pub op: Operation,
    pub start: usize,
    pub end: usize,
}

// Wing-Gong like checker
pub fn CheckLinearizable(history: &ExecutionHistory) -> bool {
    let mut keys_history: HashMap<Key, Vec<Call>> = HashMap::new();
    let mut max_time = 0;

    for entry in history {
        if let Some(call) = ParseEntry(entry) {
            max_time = max_time.max(call.end);
            keys_history.entry(call.key).or_default().push(call);
        }
    }

    if keys_history.is_empty() {
        return true;
    }

    for (key, mut ops) in keys_history {
        // Fix for Incomplete Operations
        // Identify values that were read but never logged as a finished 'Put'
        let mut written_values = HashSet::new();
        let mut read_values = HashSet::new();
        for op in &ops {
            match op.op {
                Operation::Write(v) => {
                    written_values.insert(v);
                }
                Operation::Read(v) => {
                    if v != 0 {
                        read_values.insert(v);
                    }
                }
            }
        }

        let missing_values: Vec<Value> = read_values.difference(&written_values).cloned().collect();

        if !missing_values.is_empty() {
            println!(
                "Key {}: Found {} values read from unlogged/pending writes: {:?}",
                key,
                missing_values.len(),
                missing_values
            );

            for v in missing_values {
                // Synthesize a Write that must have happened.
                // We know it must have finished before the first Read that saw it.
                let first_read_end = ops
                    .iter()
                    .filter(|o| matches!(o.op, Operation::Read(rv) if rv == v))
                    .map(|o| o.end)
                    .min()
                    .unwrap_or(max_time);

                ops.push(Call {
                    key,
                    op: Operation::Write(v),
                    start: 0, // We don't know when it started
                    end: first_read_end,
                });
            }
        }

        ops.sort_by_key(|op| op.end);
        if !CheckSingleKey(&ops) {
            println!("Linearizability violation for key {}!", key);
            return false;
        }
    }

    println!("Checker: History is linearizable!");
    true
}

fn ParseEntry(entry: &crate::abd_store::client::ExecutionHistoryEntry) -> Option<Call> {
    let op_str = entry.operation.replace(" ", "");

    if op_str.starts_with("Get") {
        let key_str = op_str.strip_prefix("Get(")?.strip_suffix(")")?;
        let key: Key = key_str.parse().ok()?;
        let value = entry.result?;
        Some(Call {
            key,
            op: Operation::Read(value),
            start: entry.start.0,
            end: entry.end.0,
        })
    } else if op_str.starts_with("Put") {
        let inner = op_str.strip_prefix("Put(")?.strip_suffix(")")?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 2 {
            return None;
        }
        let key: Key = parts[0].parse().ok()?;
        let value: Value = parts[1].parse().ok()?;
        Some(Call {
            key,
            op: Operation::Write(value),
            start: entry.start.0,
            end: entry.end.0,
        })
    } else {
        None
    }
}

fn CheckSingleKey(ops: &[Call]) -> bool {
    let mut used = vec![false; ops.len()];
    Search(ops, &mut used, 0, 0)
}

fn Search(ops: &[Call], used: &mut [bool], count: usize, current_value: Value) -> bool {
    if count == ops.len() {
        return true;
    }

    let mut min_end = usize::MAX;
    for i in 0..ops.len() {
        if !used[i] && ops[i].end < min_end {
            min_end = ops[i].end;
        }
    }

    for i in 0..ops.len() {
        if used[i] {
            continue;
        }
        let op = &ops[i];

        if op.start > min_end {
            continue;
        }

        let consistent = match op.op {
            Operation::Read(v) => v == current_value,
            Operation::Write(_) => true,
        };

        if consistent {
            used[i] = true;
            let next_value = match op.op {
                Operation::Read(_) => current_value,
                Operation::Write(v) => v,
            };

            if Search(ops, used, count + 1, next_value) {
                return true;
            }
            used[i] = false;
        }
    }

    false
}
