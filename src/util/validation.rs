use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
        .expect("compile email regex")
});
const USERNAME_MAX: usize = 30;
static USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9\.\-_]*[A-Za-z0-9]$").unwrap());

const PASSWORD_MIN: usize = 12;
const PASSWORD_MAX: usize = 128;

/// Unlike from [`validator::validate_email`], this function
/// validates email but it only prohibits host names that are in IP address
/// form (whether may be IPv4 or IPv6).
///
/// This is to prevent from unwanted self-server attacks and
/// self-sending emails (if it is assigned to a private IP address
/// where the Whim instance is hosting from, for example).
///
/// If you're trying to sign up then it turns out your email address's host name
/// only has an IP address, we do recommend purchasing a domain, or signing up
/// to any free email providers and replace it from invalid one.
///
/// [`validate_email`]: validator::validate_email
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email) && email.len() <= 254
}

pub fn is_valid_password(pass: &str) -> bool {
    let len = pass.len();
    (PASSWORD_MIN..=PASSWORD_MAX).contains(&len)
}

pub fn is_valid_username(name: &str) -> bool {
    USERNAME_REGEX.is_match(name) && name.len() <= USERNAME_MAX
}

#[cfg(test)]
mod tests {
    use super::{is_valid_email, is_valid_username};

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("gush@gmail.com"));
        assert!(!is_valid_email("nada_neutho"));
    }

    #[test]
    fn test_is_valid_username() {
        assert!(is_valid_username("memothelemo"));
        assert!(is_valid_username("mark.robes"));
        assert!(is_valid_username("salmon-ella"));
        assert!(is_valid_username("crossword_puzzle"));
        assert!(is_valid_username("slime_lover.123"));
        assert!(is_valid_username("1-taylor.swift.fan"));
        assert!(is_valid_username("2pac"));
        assert!(is_valid_username("_apple"));

        assert!(!is_valid_username("overlover_underscore_"));
        assert!(!is_valid_username("pretty ugly"));
    }
}
