use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::{InstanceSettings, User};

/// Checks whether a user is an administrator of a Capwat instance.
pub fn check_if_admin(user: &User) -> Result<(), ApiError> {
    if user.admin {
        Ok(())
    } else {
        Err(ApiError::new(ApiErrorCategory::AccessDenied))
    }
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
