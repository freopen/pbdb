use std::{
  marker::PhantomData,
  path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PbdbError {
  #[error("RocksDB error")]
  RocksDB(#[from] rocksdb::Error),
  #[error("Protobuf decode error")]
  ProtoDecodeError(#[from] prost::DecodeError),
  #[error("Collection is not registered: {0}")]
  CollectionNotRegistered(&'static str),
}

pub type PbdbResult<T> = Result<T, PbdbError>;

pub struct Id<T: Collection>(Vec<u8>, PhantomData<T>);

impl<T: Collection> Id<T> {
  pub fn new(id: Vec<u8>) -> Id<T> {
    Id::<T>(id, PhantomData)
  }
}

pub trait Collection: prost::Message + Default {
  const CF_NAME: &'static str;

  fn id(&self) -> Id<Self>;
}

pub struct DatabaseOptions {
  opts: rocksdb::Options,
  path: PathBuf,
  cf_descriptors: Vec<rocksdb::ColumnFamilyDescriptor>,
}

impl std::ops::Deref for DatabaseOptions {
  type Target = rocksdb::Options;

  fn deref(&self) -> &Self::Target {
    &self.opts
  }
}

impl std::ops::DerefMut for DatabaseOptions {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.opts
  }
}

impl DatabaseOptions {
  pub fn new(path: &Path) -> Self {
    DatabaseOptions {
      opts: rocksdb::Options::default(),
      path: path.to_path_buf(),
      cf_descriptors: vec![],
    }
  }

  pub fn register_collection<T: Collection>(&mut self) {
    self
      .cf_descriptors
      .push(rocksdb::ColumnFamilyDescriptor::new(
        T::CF_NAME,
        rocksdb::Options::default(),
      ));
  }

  pub fn open(self) -> PbdbResult<Database> {
    Ok(Database {
      rocks: rocksdb::DB::open_cf_descriptors(&self.opts, self.path, self.cf_descriptors)?,
    })
  }
}

pub struct Database {
  rocks: rocksdb::DB,
}

impl Database {
  fn cf_handle<T: Collection>(&self) -> PbdbResult<&rocksdb::ColumnFamily> {
    match self.rocks.cf_handle(T::CF_NAME) {
      Some(cf) => Ok(cf),
      None => Err(PbdbError::CollectionNotRegistered(T::CF_NAME)),
    }
  }

  pub fn get<T: Collection>(&self, id: &Id<T>) -> PbdbResult<Option<T>> {
    Ok(
      self
        .rocks
        .get_cf(self.cf_handle::<T>()?, &id.0)?
        .map(|buf| T::decode(&*buf))
        .transpose()?,
    )
  }

  pub fn put<T: Collection>(&self, value: &T) -> PbdbResult<()> {
    self
      .rocks
      .put_cf(self.cf_handle::<T>()?, value.id().0, value.encode_to_vec())?;
    Ok(())
  }

  pub fn delete<T: Collection>(&self, id: &Id<T>) -> PbdbResult<()> {
    self.rocks.delete_cf(self.cf_handle::<T>()?, &id.0)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use prost::Message;
  use tempfile::tempdir;

  #[derive(Clone, PartialEq, Message)]
  struct BasicMessage {
    #[prost(string)]
    pub id: String,
    #[prost(uint32)]
    pub value: u32,
  }

  impl BasicMessage {
    fn id(id: &str) -> Id<Self> {
      Id::new(id.as_bytes().to_owned())
    }
  }

  impl Collection for BasicMessage {
    const CF_NAME: &'static str = "basic_message";

    fn id(&self) -> Id<Self> {
      Id::new(self.id.as_bytes().to_owned())
    }
  }

  #[test]
  fn basic_database() {
    let dir = tempdir().expect("Failed to create temp dir");
    let msg = BasicMessage {
      id: "test".to_string(),
      value: 2,
    };
    {
      let mut db_options = DatabaseOptions::new(dir.path());
      db_options.create_if_missing(true);
      db_options.create_missing_column_families(true);
      db_options.register_collection::<BasicMessage>();
      let db = db_options.open().expect("Failed to open db");
      db.put(&msg).expect("Failed to put message");
    }
    {
      let mut db_options = DatabaseOptions::new(dir.path());
      db_options.register_collection::<BasicMessage>();
      let db = db_options.open().expect("Failed to open db");
      assert_eq!(
        Some(msg),
        db.get(&BasicMessage::id("test"))
          .expect("Failed to get message")
      );
      db.delete(&BasicMessage::id("test"))
        .expect("Failed to delete message");
      assert_eq!(
        None,
        db.get(&BasicMessage::id("test"))
          .expect("Failed to get message")
      );
    }
  }
}
