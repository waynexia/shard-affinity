//! Threading Load Runner
#![feature(test)]

use futures::future::join_all;
use load::ThreadingLoad;
use rand::random;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

const CONCURRENT_NUM: usize = 128 * 4;
const WRITE_BATCH_SIZE: usize = 4096 * 4;
const WRITE_LOOP_NUM: usize = 1024;
const READ_BATCH_SIZE: usize = 4096 * 4;
const READ_LOOP_NUM: usize = 1024 * 4;
const MAX_ID: usize = 1024 * 2;

fn main() {
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(16)
        .build()
        .unwrap();
    let load = Arc::new(ThreadingLoad::new());

    let now = Instant::now();
    for _ in 0..WRITE_LOOP_NUM {
        let mut write_handles = vec![];
        let bytes: Vec<u8> = (0..WRITE_BATCH_SIZE).map(|_| random()).collect();
        for _ in 0..CONCURRENT_NUM {
            let bytes = bytes.clone();
            let load = load.clone();
            write_handles.push(rt.spawn(async move {
                let id = random::<usize>() % MAX_ID;
                load.append(id, bytes);
            }));
        }
        rt.block_on(join_all(write_handles));
    }
    println!("write cost {} ms", now.elapsed().as_millis());

    let prof_guard = pprof::ProfilerGuard::new(100).unwrap();
    let now = Instant::now();
    for _ in 0..READ_LOOP_NUM {
        let mut read_handles = vec![];
        for _ in 0..CONCURRENT_NUM {
            let load = load.clone();
            read_handles.push(rt.spawn(async move {
                let id = random::<usize>() % MAX_ID;
                let _ = load.get(id, READ_BATCH_SIZE);
            }));
        }
        rt.block_on(join_all(read_handles));
    }
    println!("read cost {} ms", now.elapsed().as_millis());
    if let Ok(report) = prof_guard.report().build() {
        let file = std::fs::File::create("threading_flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
