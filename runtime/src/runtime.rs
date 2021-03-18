use core_affinity::CoreId;
use std::future::Future;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::task::RcTask;

pub struct Runtime {
    queues: Vec<Sender<RcTask>>,
}

impl Runtime {
    pub fn new(core_ids: &[CoreId]) -> Self {
        let mut queues = Vec::with_capacity(core_ids.len());
        for core_id in core_ids {
            let (tx, rx) = channel::<RcTask>();
            queues.push(tx);
            let core_id = core_id.to_owned();
            thread::spawn(move || {
                core_affinity::set_for_current(core_id);
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
