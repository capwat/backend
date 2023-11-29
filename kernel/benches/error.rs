use bencher::{benchmark_group, benchmark_main, black_box, Bencher};
use capwat_kernel::error::Error;
use capwat_types::error::{ErrorCode, ErrorType, RawError};
use serde_value::Value;
use std::collections::BTreeMap;

fn to_tonic_status_simple(b: &mut Bencher) {
  let error = black_box(Error::new(ErrorType::Internal));
  b.iter(|| error.into_tonic_status());
}

fn to_tonic_status_complex(b: &mut Bencher) {
  let mut map = BTreeMap::new();
  map.insert(Value::String("name".into()), Value::String("memo".into()));
  map.insert(Value::String("age".into()), Value::String("<unknown>".into()));

  let value = black_box(Error::new(ErrorType::Unknown(RawError {
    code: ErrorCode::Unknown(100_000),
    subcode: Some(25),
    message: "Hello world, this is to test the serialization".into(),
    data: Some(Value::Map(map)),
  })));
  b.iter(|| value.into_tonic_status());
}

benchmark_group!(benches, to_tonic_status_simple, to_tonic_status_complex);
benchmark_main!(benches);
