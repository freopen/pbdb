use std::io::Write;
use thiserror::Error;

use private::DB;

pub mod private;
pub use pbdb_macros::pbdb_impls;

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  PbdbError(String),
  #[error("RocksDB error: {0}")]
  RocksdbError(#[from] rocksdb::Error),
  #[error("Protobuf decoding error: {0}")]
  ProstDecodeError(#[from] prost::DecodeError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Collection: prost::Message + Default {
  const CF_NAME: &'static str;
  type Id;
  type SerializedId: AsRef<[u8]>;

  fn get_id(&self) -> Self::SerializedId;

  fn build_id(id: &Self::Id) -> Self::SerializedId;

  fn open_cf<'a>(
    read: &'a parking_lot::RwLockReadGuard<Option<rocksdb::DB>>,
  ) -> Result<(&'a rocksdb::DB, &'a rocksdb::ColumnFamily)> {
    let db = read
      .as_ref()
      .ok_or_else(|| Error::PbdbError("Pbdb database not initialized".to_string()))?;
    let cf = db.cf_handle(Self::CF_NAME).ok_or_else(|| {
      Error::PbdbError(format!(
        "(INTERNAL ERROR) Column family {} not found",
        Self::CF_NAME
      ))
    })?;
    Ok((db, cf))
  }

  fn get(id: &Self::Id) -> Result<Option<Self>> {
    let read = DB.read();
    let (db, cf) = Self::open_cf(&read)?;
    let from_db = db.get_pinned_cf(cf, Self::build_id(id))?;
    Ok(from_db.map(|raw| Self::decode(&*raw)).transpose()?)
  }

  fn put(&self) -> Result<()> {
    let read = DB.read();
    let (db, cf) = Self::open_cf(&read)?;
    db.put_cf(cf, Self::get_id(self), self.encode_to_vec())?;
    Ok(())
  }

  fn delete(id: &Self::Id) -> Result<()> {
    let read = DB.read();
    let (db, cf) = Self::open_cf(&read)?;
    db.delete_cf(cf, Self::build_id(id))?;
    Ok(())
  }
}

pub trait SingleRecord: prost::Message + Default {
  const RECORD_ID: &'static str;

  fn open_cf<'a>(
    read: &'a parking_lot::RwLockReadGuard<Option<rocksdb::DB>>,
  ) -> Result<(&'a rocksdb::DB, &'a rocksdb::ColumnFamily)> {
    let db = read
      .as_ref()
      .ok_or_else(|| Error::PbdbError("Pbdb database not initialized".to_string()))?;
    let cf = db.cf_handle("__SingleRecord").ok_or_else(|| {
      Error::PbdbError("(INTERNAL ERROR) Column family for single records not found".to_string())
    })?;
    Ok((db, cf))
  }

  fn get() -> Result<Self> {
    let read = DB.read();
    let (db, cf) = Self::open_cf(&read)?;
    let from_db = db.get_pinned_cf(cf, Self::RECORD_ID)?;
    Ok(
      from_db
        .map(|raw| Self::decode(&*raw))
        .transpose()?
        .unwrap_or_default(),
    )
  }

  fn put(&self) -> Result<()> {
    let read = DB.read();
    let (db, cf) = Self::open_cf(&read)?;
    db.put_cf(cf, Self::RECORD_ID, self.encode_to_vec())?;
    Ok(())
  }
}

pub struct DbGuard {}

impl Drop for DbGuard {
  fn drop(&mut self) {
    *DB.write() = None;
  }
}

pub fn create_pbdb_proto(path: &std::path::Path) {
  std::fs::File::create(path.join("pbdb.proto"))
    .expect("Failed to create pbdb.proto")
    .write_all(include_bytes!("pbdb.proto"))
    .expect("Failed to write pbdb.proto");
}
