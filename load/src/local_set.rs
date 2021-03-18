use cache::{Bytes, Cache, Id};

use std::rc::Rc;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};
use tokio::sync::oneshot;
use tokio::task::LocalSet;

#[derive(Debug)]
enum Task {
    Append(Id, Bytes, oneshot::Sender<()>),
    Get(Id, usize, oneshot::Sender<Option<Bytes>>),
}

struct LocalShard<const SHARD_NUM: usize> {
    rx: Receiver<Task>,
    caches: Rc<Vec<Cache>>,
}

impl<const SHARD_NUM: usize> LocalShard<SHARD_NUM> {
    pub fn new(rx: Receiver<Task>) -> Self {
        let mut caches = Vec::with_capacity(SHARD_NUM);
        caches.resize_with(SHARD_NUM, Default::default);
        let caches = Rc::new(caches);

        Self { rx, caches }
    }

    /// Block current thread to process tasks.
    pub fn run(self) {
        let local = LocalSet::new();
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        let Self { mut rx, caches } = self;

        local.spawn_local(async move {
            while let Some(task) = rx.recv().await {
                tokio::task::spawn_local(Self::run_task(caches.clone(), task));
            }
        });

        rt.block_on(local);
    }

    async fn run_task(caches: Rc<Vec<Cache>>, task: Task) {
        match task {
            Task::Append(id, bytes, tx) => {
                let bytes = calculation(bytes);
                let id = Self::shard_id(id);
                caches[id].append(id, bytes);

                tx.send(()).unwrap();
            }
            Task::Get(id, size, tx) => {
                let id = Self::shard_id(id);
                let result = caches[id].get(id, size).map(calculation);

                tx.send(result).unwrap()
            }
        }
    }

    #[inline]
    fn shard_id(id: Id) -> usize {
        id % SHARD_NUM
    }
}

const CACHE_PER_SHARD: usize = 10;
const WORKING_THREAD: usize = 15;

pub struct LocalSetLoad {
    txs: Vec<Sender<Task>>,
}

impl LocalSetLoad {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut txs = Vec::with_capacity(WORKING_THREAD);
        for _ in 0..WORKING_THREAD {
            let (tx, rx) = unbounded_channel();

            txs.push(tx);

            std::thread::spawn(move || {
                let shard = LocalShard::<CACHE_PER_SHARD>::new(rx);
                shard.run();
            });
        }

        Self { txs }
    }

    pub async fn append(&self, id: Id, bytes: Bytes) {
        let (tx, rx) = oneshot::channel();
        let task = Task::Append(id, bytes, tx);
        let shard_id = self.shard_id(id);

        self.txs[shard_id].send(task).unwrap();

        rx.await.unwrap()
    }

    pub async fn get(&self, id: Id, size: usize) {
        let (tx, rx) = oneshot::channel();
        let task = Task::Get(id, size, tx);
        let shard_id = self.shard_id(id);

        self.txs[shard_id].send(task).unwrap();

        let _ = rx.await.unwrap();
    }

    #[inline]
    fn shard_id(&self, id: Id) -> usize {
        id % WORKING_THREAD
    }
}

#[inline]
fn calculation(mut bytes: Bytes) -> Bytes {
    let mut sum: u8 = 0;
    bytes.iter().for_each(|x| sum = sum.wrapping_add(*x));
    bytes[0] += sum;
    bytes
}
