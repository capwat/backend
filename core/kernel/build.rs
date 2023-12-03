// No operation needed.
#[cfg(not(feature = "grpc"))]
fn main() {}

#[cfg(feature = "grpc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::configure().compile(
    &["../../proto/data.proto", "../../proto/schema.proto"],
    &["../../proto"],
  )?;

  println!("cargo:rerun-if-changed=../../proto/data.proto");
  println!("cargo:rerun-if-changed=../../proto/schema.proto");
  Ok(())
}
