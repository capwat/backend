use proc_macro2::TokenStream;
use quote::ToTokens;

use super::Context;

pub struct AttributeValue<'a, T> {
    ctx: &'a Context,
    name: &'static str,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'a, T> AttributeValue<'a, T> {
    #[must_use]
    pub fn new(ctx: &'a Context, name: &'static str) -> Self {
        Self { ctx, name, tokens: TokenStream::new(), value: None }
    }

    pub fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            let msg = format!("duplicated validator attribute `{}`", self.name);
            self.ctx.spanned_error(tokens, msg);
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    pub fn get(self) -> Option<T> {
        self.value
    }
}
