use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::item::Item;

pub type Timestamp = i64;
pub type Bytes = Vec<u8>;
pub type BytesRef<'a> = &'a [u8];

#[derive(Default)]
pub struct CacheCell {
    // todo: only keep [Item]'s reference.
    items: RwLock<BTreeMap<Timestamp, RwLock<Item>>>,
}

impl CacheCell {
    pub fn get(&self, timestamp: Timestamp, pos: usize) -> Option<Bytes> {
        Some(
            self.items
                .read()
                .unwrap()
                .get(&timestamp)?
                .read()
                .unwrap()
                .get(pos),
        )
    }

    pub fn append(&self, timestamp: Timestamp, bytes: Bytes) {
        self.items
            .write()
            .unwrap()
            .entry(timestamp)
            .or_default()
            .write()
            .unwrap()
            .put(bytes)
    }
}
