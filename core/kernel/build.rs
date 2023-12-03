// No operation needed.
#[cfg(not(feature = "grpc"))]
fn main() {}

macro_rules! proto_dir {
  ($($entry:tt)*) => {
    concat!("../../proto" $(, $entry)*)
  };
}

#[cfg(feature = "grpc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::configure().compile(
    &[proto_dir!("/data.proto"), proto_dir!("/schema.proto")],
    &[proto_dir!()],
  )?;

  println!(concat!("cargo:rerun-if-changed=", proto_dir!("/data.proto")));
  println!(concat!("cargo:rerun-if-changed=", proto_dir!("/schema.proto")));

  Ok(())
}
