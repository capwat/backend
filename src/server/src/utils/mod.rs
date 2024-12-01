//     use argon2::password_hash::{rand_core::OsRng, SaltString};
//     use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
//     use capwat_error::{ApiError, ApiErrorCategory, Error, Result};
//     use capwat_model::instance_settings::InstanceSettings;
//     use capwat_model::User;
//     use thiserror::Error;

//     pub fn check_email_status(
//         user: &User,
//         settings: &InstanceSettings,
//     ) -> std::result::Result<(), ApiError> {
//         // maybe that user has registered before in that instance but the
//         // instance administrator decided to require all users to have their
//         // own registered email later on.
//         if user.email.is_none() && settings.require_email_registration {
//             return Err(ApiError::new(ApiErrorCategory::NoEmailAddress)
//                 .message("Please input your email address to continue using Capwat"));
//         }

//         if user.email.is_some() && !user.email_verified && settings.require_email_verification {
//             return Err(ApiError::new(ApiErrorCategory::EmailVerificationRequired)
//                 .message("Please verify your email address to continue using Capwat"));
//         }

//         Ok(())
//     }

#[cfg(test)]
pub mod test;
