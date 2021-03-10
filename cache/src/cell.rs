use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::item::Item;

pub type Bytes = Vec<u8>;
pub type BytesRef<'a> = &'a [u8];
pub type Id = usize;

#[derive(Default)]
pub struct CacheCell {
    // todo: only keep [Item]'s reference.
    items: RwLock<BTreeMap<Id, RwLock<Item>>>,
}

impl CacheCell {
    /// Get random `size`.
    pub fn get(&self, id: Id, size: usize) -> Option<Bytes> {
        Some(
            self.items
                .read()
                .unwrap()
                .get(&id)?
                .read()
                .unwrap()
                .get(size),
        )
    }

    pub fn append(&self, id: usize, bytes: Bytes) {
        self.items
            .write()
            .unwrap()
            .entry(id)
            .or_default()
            .write()
            .unwrap()
            .put(bytes)
    }
}
