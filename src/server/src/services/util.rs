use capwat_api_types::error::category::PublishPostFailed;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::{InstanceSettings, User};

/// Checks the content of the public post.
pub fn check_post_content(
    _user: &User,
    settings: &InstanceSettings,
    content: &str,
) -> Result<(), ApiError> {
    // There are set of requirements of what makes an approriate post
    // on the server side (not on the moderator's side).
    //
    // The content of a post must not be empty or more the characters
    // set by the instance administrator.
    if content.is_empty() {
        return Err(ApiError::new(ApiErrorCategory::PublishPostFailed(
            PublishPostFailed::EmptyContent,
        )));
    }

    // Now, this is getting wild. We'll going to refer characters as how
    // many bytes are there in a single content byte array.
    if content.as_bytes().len() > settings.post_max_characters as usize {
        let message = "Too many characters!";
        return Err(ApiError::new(ApiErrorCategory::PublishPostFailed(
            PublishPostFailed::TooManyCharacters,
        ))
        .message(message));
    }

    Ok(())
}

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
