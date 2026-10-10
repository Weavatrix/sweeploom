//! Bounded fan-out for filesystem work. Scoped threads, no global pool.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Worker count for metadata-heavy walks: the CPU count, clamped so a slow
/// disk is not flooded with concurrent `stat` calls.
#[must_use]
pub fn walk_workers() -> usize {
    std::thread::available_parallelism()
        .map_or(4, std::num::NonZeroUsize::get)
        .clamp(2, 12)
}

/// Map `items` on up to `workers` scoped threads. Results keep input order.
///
/// A panic in `map` propagates to the caller once every worker has stopped.
pub fn parallel_map<T, R, F>(items: Vec<T>, workers: usize, map: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync,
{
    if items.len() <= 1 || workers <= 1 {
        return items.into_iter().map(map).collect();
    }
    let inputs: Vec<Mutex<Option<T>>> = items.into_iter().map(|item| Mutex::new(Some(item))).collect();
    let outputs: Vec<Mutex<Option<R>>> = inputs.iter().map(|_| Mutex::new(None)).collect();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..workers.min(inputs.len()) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(input) = inputs.get(index) else {
                        break;
                    };
                    let Some(item) = input.lock().ok().and_then(|mut slot| slot.take()) else {
                        continue;
                    };
                    let value = map(item);
                    if let Ok(mut slot) = outputs[index].lock() {
                        *slot = Some(value);
                    }
                }
            });
        }
    });
    outputs
        .into_iter()
        .filter_map(|slot| slot.into_inner().ok().flatten())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_keep_input_order_across_workers() {
        let items: Vec<u64> = (0..200).collect();
        let squares = parallel_map(items, 7, |value| {
            if value % 13 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            value * value
        });
        assert_eq!(squares, (0..200).map(|value| value * value).collect::<Vec<_>>());
        assert!(walk_workers() >= 2);
    }
}
