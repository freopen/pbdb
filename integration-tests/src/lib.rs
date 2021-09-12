#[cfg(test)]
mod basic_message;

mod proto {
  include!(concat!(env!("OUT_DIR"), "/tests.rs"));
  pbdb::pbdb_impls!();
}
