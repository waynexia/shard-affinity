//! Threading Load Runner
#![feature(test)]

use futures::future::join_all;
use load::ThreadingLoad;
use rand::random;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

const CONCURRENT_NUM: usize = 1024;
const WRITE_BATCH_SIZE: usize = 4096 * 2;
const WRITE_LOOP_NUM: usize = 1024;
const READ_BATCH_SIZE: usize = 4096 * 4;
const READ_LOOP_NUM: usize = 1024 * 16;
const MAX_ID: usize = 1024 * 2;

async fn write_data(load: Arc<ThreadingLoad>) -> u128 {
    let now = Instant::now();
    for _ in 0..WRITE_LOOP_NUM {
        let id = random::<usize>() % MAX_ID;
        let bytes: Vec<u8> = (0..WRITE_BATCH_SIZE).map(|_| random()).collect();
        load.append(id, bytes);
    }
    now.elapsed().as_millis()
}

async fn read_data(load: Arc<ThreadingLoad>) -> u128 {
    let now = Instant::now();
    for _ in 0..READ_LOOP_NUM {
        let id = random::<usize>() % MAX_ID;
        let _ = load.get(id, READ_BATCH_SIZE).unwrap_or_default();
    }
    now.elapsed().as_millis()
}

fn main() {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(16)
        .build()
        .unwrap();

    let load = Arc::new(ThreadingLoad::new());

    let now = Instant::now();
    let mut write_timers = vec![];
    for _ in 0..CONCURRENT_NUM {
        write_timers.push(rt.spawn(write_data(load.clone())));
    }
    println!("spawn write elapsed: {}", now.elapsed().as_micros());
    let write_timer: u128 = rt
        .block_on(join_all(write_timers))
        .into_iter()
        .map(Result::unwrap)
        .sum();

    println!("write  cost {} ms", write_timer);

    let mut read_timers = vec![];
    for _ in 0..CONCURRENT_NUM {
        read_timers.push(rt.spawn(read_data(load.clone())));
    }
    let read_timer: u128 = rt
        .block_on(join_all(read_timers))
        .into_iter()
        .map(Result::unwrap)
        .sum();

    println!("read cost {} ms", read_timer);
}
