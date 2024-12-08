use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::{instance::InstanceSettings, user::UserKeys, User};
use chrono::{DateTime, Utc};

/// Checks whether the user's public/private keys is/are okay to use
/// based on its expiration timestamp from the timestamp given.
///
/// It will throw an error if it actually is expired.
pub fn check_user_keys(their_keys: &UserKeys, now: DateTime<Utc>) -> Result<(), ApiError> {
    if now.naive_utc() > their_keys.expires_at {
        return Err(ApiError::new(ApiErrorCategory::KeysExpired)
            .message("Your current public and private subkeys are expired. Please renew your keys to continue using Capwat."));
    }

    Ok(())
}

/// Checks whether a user successfully verifies their email address
/// if required from the local instance settings.
///
/// It will throw an error if the user hasn't verified their email
/// address and email verification is required in the local instance
/// settings.
pub fn check_email_status(user: &User, settings: &InstanceSettings) -> Result<(), ApiError> {
    // Maybe that user has registered before in that instance but the
    // instance administrator decided to require all users to have their
    // own registered email later on.
    if user.email.is_none() && settings.require_email_registration {
        return Err(ApiError::new(ApiErrorCategory::NoEmailAddress)
            .message("Please input your email address to continue using Capwat"));
    }

    if user.email.is_some() && !user.email_verified && settings.require_email_verification {
        return Err(ApiError::new(ApiErrorCategory::EmailVerificationRequired)
            .message("Please verify your email address to continue using Capwat"));
    }

    Ok(())
}
