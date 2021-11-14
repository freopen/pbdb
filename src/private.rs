use parking_lot::{RwLock, const_rwlock};

pub use rocksdb;

pub static DB: RwLock<Option<rocksdb::DB>> = const_rwlock(None);
