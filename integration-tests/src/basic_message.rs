use pbdb::Collection;
use tempfile::tempdir;

mod proto {
  include!(concat!(env!("OUT_DIR"), "/tests.rs"));
  pbdb::pbdb_impls!();
}

#[test]
fn basic_database() {
  let dir = tempdir().expect("Failed to create temp dir");
  dbg!(&dir);
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
