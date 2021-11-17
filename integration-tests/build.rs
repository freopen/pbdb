use std::{env, io::Result, path::PathBuf};

fn main() -> Result<()> {
  pbdb::create_pbdb_proto(std::path::Path::new("proto"));
  prost_build::Config::new()
    .file_descriptor_set_path(
      PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
        .join("file_descriptor_set.bin"),
    )
    .compile_protos(&["proto/tests.proto"], &["proto"])?;
  Ok(())
}
