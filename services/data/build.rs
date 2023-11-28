use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  tonic_build::compile_protos("../../proto/data.proto")?;
  tonic_build::compile_protos("../../proto/schema.proto")?;

  Ok(())
}
