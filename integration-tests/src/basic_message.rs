use pbdb::DatabaseOptions;
use tempfile::tempdir;

mod proto {
  include!(concat!(env!("OUT_DIR"), "/tests.rs"));
  pbdb::pbdb_impls!();
}

#[test]
fn basic_database() {
  let dir = tempdir().expect("Failed to create temp dir");
  let msg = proto::BasicMessage {
    id: "test".to_string(),
    value: 2,
  };
  {
    let mut db_options = DatabaseOptions::new(dir.path());
    db_options.create_if_missing(true);
    db_options.create_missing_column_families(true);
    db_options.register_collection::<proto::BasicMessage>();
    let db = db_options.open().expect("Failed to open db");
    db.put(&msg).expect("Failed to put message");
  }
  {
    let mut db_options = DatabaseOptions::new(dir.path());
    db_options.register_collection::<proto::BasicMessage>();
    let db = db_options.open().expect("Failed to open db");
    assert_eq!(
      Some(msg),
      db.get(&proto::BasicMessage::id("test"))
        .expect("Failed to get message")
    );
    db.delete(&proto::BasicMessage::id("test"))
      .expect("Failed to delete message");
    assert_eq!(
      None,
      db.get(&proto::BasicMessage::id("test"))
        .expect("Failed to get message")
    );
  }
}
