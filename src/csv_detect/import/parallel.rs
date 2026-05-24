use super::*;
use anyhow::anyhow;
use std::ops::Range;

pub(super) fn run_indexed_in_parallel<T, F>(
    item_count: usize,
    operation: F,
    panic_message: &'static str,
) -> Result<Vec<(usize, Result<T>)>>
where
    T: Send,
    F: Fn(usize) -> Result<T> + Sync,
{
    let worker_count = parallel_worker_count(item_count);
    if worker_count <= 1 {
        return Ok((0..item_count)
            .map(|index| (index, operation(index)))
            .collect());
    }

    std::thread::scope(|scope| {
        let operation = &operation;
        let mut handles = Vec::new();
        for range in parallel_chunk_ranges(item_count, worker_count) {
            handles.push(scope.spawn(move || {
                range
                    .map(|index| (index, operation(index)))
                    .collect::<Vec<_>>()
            }));
        }

        let mut results = Vec::with_capacity(item_count);
        for handle in handles {
            results.extend(handle.join().map_err(|_| anyhow!(panic_message))?);
        }
        results.sort_by_key(|(index, _)| *index);
        Ok(results)
    })
}

fn parallel_worker_count(item_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    item_count.min(available).max(1)
}

pub(in crate::csv_detect) fn parallel_chunk_ranges(
    item_count: usize,
    worker_count: usize,
) -> Vec<Range<usize>> {
    if item_count == 0 || worker_count == 0 {
        return Vec::new();
    }

    let worker_count = worker_count.min(item_count);
    let base = item_count / worker_count;
    let remainder = item_count % worker_count;
    let mut ranges = Vec::with_capacity(worker_count);
    let mut start = 0;
    for worker_index in 0..worker_count {
        let extra = usize::from(worker_index < remainder);
        let end = start + base + extra;
        ranges.push(start..end);
        start = end;
    }
    ranges
}
