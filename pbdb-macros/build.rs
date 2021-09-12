use std::io::Result;

fn main() -> Result<()> {
  println!("cargo:rerun-if-changed=proto/descriptor_subset.proto");
  prost_build::compile_protos(&["proto/descriptor_subset.proto"], &["proto"])?;
  Ok(())
}
