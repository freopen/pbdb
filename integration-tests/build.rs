use std::{env, io::Result, path::PathBuf};

fn main() -> Result<()> {
  prost_build::Config::new()
    .file_descriptor_set_path(
      PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
        .join("file_descriptor_set.bin"),
    )
    .compile_protos(&["proto/test.proto"], &["proto", "../proto"])?;
  Ok(())
}
