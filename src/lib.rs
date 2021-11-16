pub mod private;
use private::DB;

pub use pbdb_macros::pbdb_impls;

pub trait Collection: prost::Message + Default {
  const CF_NAME: &'static str;
  type Id;
  type SerializedId: AsRef<[u8]>;

  fn get_id(&self) -> Self::SerializedId;

  fn build_id(id: &Self::Id) -> Self::SerializedId;

  fn get(id: &Self::Id) -> Option<Self> {
    let read = DB.read();
    let db = read.as_ref().expect("Pbdb database not initialized");
    db.get_pinned_cf(db.cf_handle(Self::CF_NAME).unwrap(), Self::build_id(id))
      .unwrap()
      .map(|buf| Self::decode(&*buf).unwrap())
  }

  fn put(&self) {
    let read = DB.read();
    let db = read.as_ref().expect("Pbdb database not initialized");
    db.put_cf(
      db.cf_handle(Self::CF_NAME).unwrap(),
      Self::get_id(self),
      self.encode_to_vec(),
    )
    .unwrap()
  }

  fn delete(id: &Self::Id) {
    let read = DB.read();
    let db = read.as_ref().expect("Pbdb database not initialized");
    db.delete_cf(db.cf_handle(Self::CF_NAME).unwrap(), Self::build_id(id))
      .unwrap()
  }
}

pub trait SingleRecord: prost::Message + Default {
  const RECORD_ID: &'static str;

  fn get() -> Self {
    let read = DB.read();
    let db = read.as_ref().expect("Pbdb database not initialized");
    db.get_pinned_cf(db.cf_handle("__SingleRecord").unwrap(), Self::RECORD_ID)
      .unwrap()
      .map(|buf| Self::decode(&*buf).unwrap())
      .unwrap_or_default()
  }

  fn put(&self) {
    let read = DB.read();
    let db = read.as_ref().expect("Pbdb database not initialized");
    db.put_cf(
      db.cf_handle("__SingleRecord").unwrap(),
      Self::RECORD_ID,
      self.encode_to_vec(),
    )
    .unwrap()
  }
}

pub struct DbGuard {}

impl Drop for DbGuard {
  fn drop(&mut self) {
    *DB.write() = None;
  }
}
