use pbdb::{Collection, SingleRecord};
use tempfile::tempdir;

mod proto {
  include!(concat!(env!("OUT_DIR"), "/tests.rs"));
  pbdb::pbdb_impls!();
}

#[test]
fn basic_message() {
  let dir = tempdir().expect("Failed to create temp dir");
  let msg = proto::BasicMessage {
    id: "test".to_string(),
    value: 2,
  };
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(None, proto::BasicMessage::get("test"));
    msg.put();
    assert_eq!(Some(msg.clone()), proto::BasicMessage::get("test"));
  }
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(Some(msg), proto::BasicMessage::get("test"));
    proto::BasicMessage::delete("test");
    assert_eq!(None, proto::BasicMessage::get("test"));
  }
}

#[test]
fn single_record() {
  let dir = tempdir().expect("Failed to create temp dir");
  let msg = proto::SingleRecord {
    value: 2,
  };
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(proto::SingleRecord::default(), proto::SingleRecord::get());
    msg.put();
    assert_eq!(msg, proto::SingleRecord::get());
  }
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(msg, proto::SingleRecord::get());
  }
}
