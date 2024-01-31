use crate::Result;
use async_trait::async_trait;

/// This driver allows to sanitize user-generated content that may
/// pose as a security risk for both the server and the client.
#[async_trait]
pub trait Sanitizer: Sync + Send {
    /// Sanitizes user-generated text through series of steps
    /// in order to avoid any security risk that may be dangerous
    /// for both the server and the client.
    ///
    /// There are series of steps in ensure the sanitizer will
    /// sanitize user-generated text:
    /// - Sanitizing any HTML content to avoid [XSS (cross-site scripting attack)]
    /// - Sanitizing any possible [SQLi (SQL injection)] related text.
    ///
    /// [XSS (cross-site scripting attack)]: https://en.wikipedia.org/wiki/Cross-site_scripting
    /// [SQLi (SQL injection)]: https://en.wikipedia.org/wiki/SQL_injection
    async fn sanitize_text(&self, input: &str) -> Result<String> {
        let html = self.sanitize_html(input).await?;
        Ok(html)
    }

    /// Sanitizes user-generated text through the HTML sanitizer.
    async fn sanitize_html(&self, input: &str) -> Result<String>;
}
