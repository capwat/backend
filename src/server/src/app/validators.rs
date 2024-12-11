use validator::ValidateEmail;

use super::App;

impl App {
    /// Validates user's name.
    ///
    /// There are rules that a user must abide when choosing their
    /// own username for Capwat:
    /// - All characters must be in the English alphabet, `-` or `_`
    ///   (symbols are not allowed in the first character of the name)
    /// - The length of the user's name must be within 3 to 20 characters
    #[must_use]
    pub fn validate_username(&self, name: &str) -> bool {
        fn is_validate_username_char(c: char) -> bool {
            c.is_ascii_alphabetic() || matches!(c, '-' | '_')
        }

        let has_valid_chars = name.chars().all(is_validate_username_char);
        let must_not_start_with_symbols = name
            .chars()
            .next()
            .map(|v| v.is_ascii_alphabetic())
            .unwrap_or_default();

        let has_valid_length = (3..=20).contains(&name.len());
        has_valid_chars && must_not_start_with_symbols && has_valid_length
    }

    /// Validates user's email address
    #[must_use]
    pub fn validate_email(&self, email: &str) -> bool {
        !email.is_empty() && email.validate_email() && email.to_lowercase() == email
    }
}
