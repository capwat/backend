use error_stack::Report;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Got an error")]
struct TestError;

#[derive(Debug, Serialize)]
struct Error {
  #[serde(serialize_with = "capwat_kernel::util::report::serialize_report")]
  error: Report<TestError>,
}

fn main() {
  let error = Error {
    error: Report::new(TestError).attach_printable("Hi").attach("Oops!"),
  };
  println!("{}", serde_json::to_string_pretty(&error).unwrap());
}
