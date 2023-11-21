use crate::{
  types::validation::{self, is_valid_email, is_valid_username},
  util::Sensitive,
};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateError};

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
  pub username: Sensitive<String>,
  pub email: Option<Sensitive<String>>,
  pub password: Sensitive<String>,
  pub confirm_password: Sensitive<String>,
}

impl Validate for Request {
  fn validate(&self) -> Result<(), ValidateError> {
    let mut fields = ValidateError::field_builder();
    fields.insert("username", {
      let mut error = ValidateError::msg_builder();
      if !is_valid_username(&self.username) {
        error.insert("Invalid username");
      }
      error.build()
    });

    if let Some(email) = self.email.as_deref() {
      fields.insert("email", {
        let mut error = ValidateError::msg_builder();
        if !is_valid_email(email) {
          error.insert("Invalid e-mail address");
        }
        error.build()
      });
    }

    // TODO: check for weak passwords
    fields.insert("password", {
      // All passwords must have no trailing or leading whitespaces
      let mut error = ValidateError::msg_builder();
      let password = self.password.as_str().trim();
      if self.password.len() != password.len() {
        error.insert("Passwords must not have starting or ending with spaces");
      } else if self.password.len() > validation::PASSWORD_MAX {
        error.insert("Passwords must not be too big");
      } else if self.password.len() < validation::PASSWORD_MIN {
        error.insert("Passwords must not be too short");
      }
      error.build()
    });

    // Not very secure... :(
    if self.password.as_str() != self.confirm_password.as_str() {
      let mut error = ValidateError::msg_builder();
      error.insert("Unmatched password");
      fields.insert("confirm_password", error.build());
    }

    fields.build().into_result()
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
  // For e-mails only and verification is required depending
  // on the feelings of the Whim instance maintainer.
  pub verification_required: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[track_caller]
  fn must_fail<T: Validate>(value: &T, args: std::fmt::Arguments<'_>) {
    if value.validate().is_ok() {
      panic!("expected to fail but passed (entry = {args})");
    }
  }

  #[test]
  fn test_password_fields() {
    static INVALID_PASSWORDS: &'static [&'static str] = &[
      "\thelloworld",
      "    hello",
      "world    ",
      "too_short",
      "we_dont_accept_tabs\t",
      concat!(
        "thisistoolongpleasedontactuallydothisathhomeotherwiseyoulldiefromtypingtoomuch",
        "imeanitdoyouknowaboutrsi?nope,ok.12345678901234567890"
      ),
    ];

    for combination in INVALID_PASSWORDS {
      let form = Request {
        username: "memothelemo".to_string().into(),
        email: None,
        password: combination.to_string().into(),
        confirm_password: combination.to_string().into(),
      };

      must_fail(&form, format_args!("{combination:?}"));
    }

    let form = Request {
      username: "memothelemo".to_string().into(),
      email: None,
      password: "wrong_password".to_string().into(),
      confirm_password: "wrong_password1".to_string().into(),
    };
    assert!(form.validate().is_err());

    let form = Request {
      username: "memothelemo".to_string().into(),
      email: None,
      password: "wrong_password".to_string().into(),
      confirm_password: "wrong_password".to_string().into(),
    };
    assert!(form.validate().is_ok());
  }
}
