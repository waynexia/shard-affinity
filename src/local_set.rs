//! Threading Load Runner
#![feature(test)]
#![feature(maybe_uninit_uninit_array)]

use load::LocalSetLoad;
use rand::random;
use std::rc::Rc;
use std::time::Instant;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

use shard_affinity::*;

fn main() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let load = Rc::new(LocalSetLoad::new());

    let now = Instant::now();
    for _ in 0..WRITE_LOOP_NUM {
        let local = LocalSet::new();
        let bytes: Vec<u8> = (0..WRITE_BATCH_SIZE).map(|_| random()).collect();
        for _ in 0..CONCURRENT_NUM {
            let id = random::<usize>() % MAX_ID;
            let bytes = bytes.clone();
            let load = load.clone();
            local.spawn_local(async move { load.append(id, bytes).await });
        }
        rt.block_on(local);
    }
    println!("write cost {} ms", now.elapsed().as_millis());

    let prof_guard = pprof::ProfilerGuard::new(100).unwrap();
    let now = Instant::now();
    for _ in 0..READ_LOOP_NUM {
        let local = LocalSet::new();
        for _ in 0..CONCURRENT_NUM {
            let id = random::<usize>() % MAX_ID;
            let load = load.clone();
            local.spawn_local(async move { load.get(id, READ_BATCH_SIZE).await });
        }
        rt.block_on(local);
    }
    println!("read cost {} ms", now.elapsed().as_millis());

    if let Ok(report) = prof_guard.report().build() {
        let _ = std::fs::create_dir("flamegraph");
        let file = std::fs::File::create("flamegraph/local_set.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
