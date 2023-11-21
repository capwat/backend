use crate::{HasLength, Validate, ValidateError};
use std::{
  borrow::Cow,
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

impl<K, V> HasLength for BTreeMap<K, V> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<V> HasLength for BTreeSet<V> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<'a> HasLength for Cow<'a, str> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<K, V, S> HasLength for HashMap<K, V, S> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<V, S> HasLength for HashSet<V, S> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<T, const N: usize> HasLength for [T; N] {
  fn length(&self) -> usize {
    // Why not, it is faster to do this way
    N
  }
}

impl<'a> HasLength for &'a String {
  fn length(&self) -> usize {
    self.len()
  }
}

impl HasLength for String {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<'a> HasLength for &'a str {
  fn length(&self) -> usize {
    self.len()
  }
}

impl HasLength for str {
  fn length(&self) -> usize {
    self.len()
  }
}

// ------------------------------------------------ //

macro_rules! validate_array {
  ($self:expr) => {{
    let mut slice = ValidateError::slice_builder();
    for element in $self.iter() {
      if let Err(err) = element.validate() {
        slice.insert(err);
      } else {
        slice.insert_empty();
      }
    }
    slice.build().into_result()
  }};
}

impl<T> HasLength for Vec<T> {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<T> HasLength for [T] {
  fn length(&self) -> usize {
    self.len()
  }
}

impl<T> Validate for [T]
where
  T: Validate,
{
  fn validate(&self) -> Result<(), ValidateError> {
    validate_array!(self)
  }
}

impl<T> Validate for Vec<T>
where
  T: Validate,
{
  fn validate(&self) -> Result<(), ValidateError> {
    validate_array!(self)
  }
}

// impl<'a, T> Validate for &'a Vec<T>
// where
//   T: Validate,
// {
//   fn validate(&self) -> Result<(), ValidateError> {
//     validate_array!(self)
//   }
// }

macro_rules! validate_map {
  ($self:expr) => {{
    let mut fields = ValidateError::field_builder();
    for (k, v) in $self.iter() {
      if let Err(err) = v.validate() {
        fields.insert(k.as_ref().to_string(), err);
      }
    }
    fields.build().into_result()
  }};
}

impl<K: AsRef<str>, V: Validate> Validate for BTreeMap<K, V> {
  fn validate(&self) -> Result<(), ValidateError> {
    validate_map!(self)
  }
}

impl<T: Validate> Validate for BTreeSet<T> {
  fn validate(&self) -> Result<(), ValidateError> {
    validate_array!(self)
  }
}

impl<'a, T: Validate + ToOwned> Validate for Cow<'a, T> {
  fn validate(&self) -> Result<(), ValidateError> {
    T::validate(self)
  }
}

impl<K: AsRef<str>, V: Validate, S> Validate for HashMap<K, V, S> {
  fn validate(&self) -> Result<(), ValidateError> {
    validate_map!(self)
  }
}

impl<T: Validate, S> Validate for HashSet<T, S> {
  fn validate(&self) -> Result<(), ValidateError> {
    validate_array!(self)
  }
}

impl<T: Validate, const N: usize> Validate for [T; N] {
  fn validate(&self) -> Result<(), ValidateError> {
    validate_array!(self)
  }
}

impl<T: Validate> Validate for Box<T> {
  fn validate(&self) -> Result<(), ValidateError> {
    T::validate(self)
  }
}

// impl<'a, T: Validate> Validate for &'a Box<T> {
//   fn validate(&self) -> Result<(), ValidateError> {
//     T::validate(&self)
//   }
// }

impl<'a, T: Validate> Validate for &'a T {
  fn validate(&self) -> Result<(), ValidateError> {
    T::validate(self)
  }
}
