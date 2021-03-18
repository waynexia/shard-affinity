use cache::{Bytes, Cache, Id};
use core_affinity::CoreId;
use runtime::Runtime;
use std::rc::Rc;
use std::thread_local;
use tokio::sync::oneshot;

const CORE_NUM: usize = 15;
const CACHE_PER_SHARD: usize = 10;

struct AffinityShard {
    caches: Rc<Vec<Cache>>,
}

impl AffinityShard {
    pub fn new() -> Self {
        let mut caches = Vec::with_capacity(CACHE_PER_SHARD);
        caches.resize_with(CACHE_PER_SHARD, Default::default);
        let caches = Rc::new(caches);

        Self { caches }
    }

    pub fn append(&self, id: Id, bytes: Bytes) {
        let bytes = calculation(bytes);
        let id = Self::shard_id(id);
        self.caches[id].append(id, bytes);
    }

    pub fn get(&self, id: Id, size: usize) -> Option<Bytes> {
        let id = Self::shard_id(id);
        self.caches[id].get(id, size).map(calculation)
    }

    fn shard_id(id: Id) -> Id {
        id % CACHE_PER_SHARD
    }
}

pub struct AffinityLoad {
    runtime: Runtime,
}

impl AffinityLoad {
    #[allow(clippy::new_without_default)]
    pub fn new(core_ids: &[CoreId]) -> Self {
        assert_eq!(core_ids.len(), CORE_NUM);
        let runtime = Runtime::new(core_ids);

        Self { runtime }
    }

    pub async fn append(&self, id: Id, bytes: Bytes) {
        let (tx, rx) = oneshot::channel();

        let id = shard_id(id);
        self.runtime.spawn(id, async move {
            thread_local! (static SHARD:AffinityShard = AffinityShard::new() );

            SHARD.with(|shard| {
                shard.append(id, bytes);
            });

            tx.send(()).unwrap();
        });

        rx.await.unwrap();
    }

    #[allow(clippy::unit_arg)]
    pub async fn get(&self, id: Id, size: usize) {
        let (tx, rx) = oneshot::channel();

        let id = shard_id(id);
        self.runtime.spawn(id, async move {
            thread_local! (static SHARD:AffinityShard = AffinityShard::new() );

            let result = SHARD.with(|shard| {
                shard.get(id, size);
            });

            tx.send(result).unwrap();
        });

        let _ = rx.await.unwrap();
    }
}

#[inline]
fn shard_id(id: Id) -> usize {
    id % CORE_NUM
}

#[inline]
fn calculation(mut bytes: Bytes) -> Bytes {
    let mut sum: u8 = 0;
    bytes.iter().for_each(|x| sum = sum.wrapping_add(*x));
    bytes[0] += sum;
    bytes
}
