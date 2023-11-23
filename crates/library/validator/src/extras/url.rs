use url::Url;

#[must_use]
pub fn validate_url(url: &str) -> bool {
  Url::parse(url).is_ok()
}
