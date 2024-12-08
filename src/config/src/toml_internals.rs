use annotate_snippets::{Level, Renderer, Snippet};
use capwat_error::Error;
use serde::de::DeserializeOwned;
use std::path::Path;
use thiserror::Error;
use toml_edit::ImDocument;

#[derive(Debug, Error)]
#[error("{0}")]
struct InnerError(String);

pub fn emit_diagnostic(e: toml_edit::de::Error, contents: &str, path: &Path) -> Error {
    let Some(span) = e.span() else {
        return e.into();
    };

    let path = path.to_string_lossy();
    let message = Level::Error.title(e.message()).snippet(
        Snippet::source(contents)
            .origin(&path)
            .fold(true)
            .annotation(Level::Error.span(span)),
    );

    let renderer = Renderer::styled();
    let message = renderer.render(message).to_string();
    Error::unknown_generic(InnerError(message))
}

pub fn parse_document(contents: &str) -> Result<ImDocument<String>, toml_edit::de::Error> {
    toml_edit::ImDocument::parse(contents.to_owned()).map_err(Into::into)
}

pub fn deserialize<T: DeserializeOwned>(
    document: &ImDocument<String>,
) -> Result<T, toml_edit::de::Error> {
    toml_edit::de::from_document(document.clone())
}
