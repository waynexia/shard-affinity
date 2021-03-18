use hwloc;
use std::future::Future;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::task::RcTask;

pub struct Runtime {
    queues: Vec<Sender<RcTask>>,
}

impl Runtime {
    pub fn new(num_thds: usize) -> Self {
        let mut queues = Vec::with_capacity(num_thds);
        for _ in 0..num_thds {
            let (tx, rx) = channel::<RcTask>();
            queues.push(tx);
            thread::spawn(move || {
                pin_thread();
                loop {
                    while let Ok(task) = rx.recv() {
                        unsafe { task.poll() }
                    }
                }
            });
        }

        Self { queues }
    }

    pub fn spawn<F>(&self, index: usize, task: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.queues[index]
            .send(RcTask::new(task, self.queues[index].clone()))
            .unwrap();
    }
}

fn pin_thread() {
    // todo!()
}
