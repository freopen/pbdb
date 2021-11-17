use pbdb::{Collection, SingleRecord};
use tempfile::tempdir;

mod proto {
  include!(concat!(env!("OUT_DIR"), "/tests.rs"));
  pbdb::pbdb_impls!();
}

#[test]
fn basic_message() {
  let dir = tempdir().expect("Failed to create temp dir");
  let id = String::from("test");
  let msg = proto::BasicMessage {
    id: id.clone(),
    value: 2,
  };
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(None, proto::BasicMessage::get(&id));
    msg.put();
    assert_eq!(Some(msg.clone()), proto::BasicMessage::get(&id));
  }
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(Some(msg), proto::BasicMessage::get(&id));
    proto::BasicMessage::delete(&id);
    assert_eq!(None, proto::BasicMessage::get(&id));
  }
}

#[test]
fn single_record() {
  let dir = tempdir().expect("Failed to create temp dir");
  let msg = proto::SingleRecord { value: 2 };
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

#[test]
fn case_insensitive() {
  let dir = tempdir().expect("Failed to create temp dir");
  let msg = proto::CaseInsensitive {
    id: String::from("test"),
  };
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("test")));
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("Test")));
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("TEST")));
    msg.put();
    assert_eq!(Some(msg.clone()), proto::CaseInsensitive::get(&String::from("test")));
    assert_eq!(Some(msg.clone()), proto::CaseInsensitive::get(&String::from("Test")));
    assert_eq!(Some(msg.clone()), proto::CaseInsensitive::get(&String::from("TEST")));
  }
  {
    let _db_guard = proto::open_db(dir.path()).unwrap();
    assert_eq!(Some(msg.clone()), proto::CaseInsensitive::get(&String::from("test")));
    assert_eq!(Some(msg.clone()), proto::CaseInsensitive::get(&String::from("Test")));
    assert_eq!(Some(msg), proto::CaseInsensitive::get(&String::from("TEST")));
    proto::CaseInsensitive::delete(&String::from("TEST"));
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("test")));
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("Test")));
    assert_eq!(None, proto::CaseInsensitive::get(&String::from("TEST")));
  }
}
