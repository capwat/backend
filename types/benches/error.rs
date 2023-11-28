#![allow(clippy::unwrap_used)]
use bencher::{benchmark_group, benchmark_main, black_box, Bencher};
use capwat_types::error::{ErrorCode, ErrorType, RawError};
use serde_value::Value;
use std::collections::BTreeMap;

fn deserialize_simple_variant(b: &mut Bencher) {
  // Message validation is usually ignored if it is a known code
  const SAMPLE: &str = r#"{"code":1,"message":""}"#;
  let sample = black_box(SAMPLE);
  b.iter(|| serde_json::from_str::<ErrorType>(sample).unwrap());
}

fn deserialize_complex_variant(b: &mut Bencher) {
  const SAMPLE: &str = concat!(
    r#"{"code":100000,"subcode":25,"message":"Hello world,"#,
    r#"this is to test the serialization","data":{"name":"memo","age":"<unknown>","interests":["#,
    r#"1,2,3,4,5,6,7,8,9,10]}}"#
  );
  let sample = black_box(SAMPLE);
  b.iter(|| serde_json::from_str::<ErrorType>(sample).unwrap());
}

fn serialize_simple_variant(b: &mut Bencher) {
  let value = black_box(ErrorType::Internal);
  b.iter(|| serde_json::to_string(&value).unwrap());
}

fn serialize_complex_variant(b: &mut Bencher) {
  let mut map = BTreeMap::new();
  map.insert(Value::String("name".into()), Value::String("memo".into()));
  map.insert(Value::String("age".into()), Value::String("<unknown>".into()));

  let value = black_box(ErrorType::Unknown(RawError {
    code: ErrorCode::Unknown(100_000),
    subcode: Some(25),
    message: "Hello world, this is to test the serialization".into(),
    data: Some(Value::Map(map)),
  }));
  b.iter(|| serde_json::to_string(&value).unwrap());
}

benchmark_group!(
  benches,
  deserialize_simple_variant,
  deserialize_complex_variant,
  serialize_simple_variant,
  serialize_complex_variant
);
benchmark_main!(benches);
