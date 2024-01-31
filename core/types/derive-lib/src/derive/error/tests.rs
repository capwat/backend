use crate::utils::tests::{extract_error_msg, parse_input};

use super::expand;

#[test]
fn it_should_not_parse_if_code_is_negative() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(code = -2)]
        One,
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!(
        r#"expected "error" attribute to be a valid integer: `code = "..."`"#,
        msg
    );
}

#[test]
fn it_should_not_use_unknown_attr_twice() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(unknown)]
        One(N),
        #[error(unknown)]
        Two(N),
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(unknown)] should not be used twice", msg);
}

#[test]
fn it_should_throw_if_any_variants_are_named() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        Hello {}
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("this macro does not support variants with named fields", msg);
}

#[test]
fn it_should_throw_if_variant_has_no_code() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        Hello
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(code = ...)] is required", msg);
}

#[test]
fn it_should_throw_if_variant_has_two_or_more_fields() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(code = 2)]
        Hello(String, u32)
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("Every error category must be either a unit variant or has one unnamed field", msg);
}

#[test]
fn it_should_throw_if_unnamed_variant_has_message_attr() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(code = 2)]
        #[error(message = "Hello")]
        Hello(String),
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(message = ...)] must be used in unit variants", msg);
}

#[test]
fn it_should_throw_if_unit_variant_has_no_message() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(code = 2)]
        Hello
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(message = ...)] is required for unit variants", msg);
}

#[test]
fn it_should_throw_if_variant_has_empty_fields() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(code = 2)]
        Hello()
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("Every error category must be either a unit variant or has one unnamed field", msg);
}

#[test]
fn it_should_throw_if_unknown_variant_has_more_than_one_unnamed_fields() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(unknown)]
        Hello
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(unknown)] variant must have one unnamed field", msg);
}

#[test]
fn it_should_throw_if_unknown_variant_has_no_unnamed_fields() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(unknown)]
        Hello
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(unknown)] variant must have one unnamed field", msg);
}

#[test]
fn it_should_throw_if_unknown_variant_has_code_attr() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(unknown)]
        #[error(code = 213)]
        Hello(N)
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!(
        "#[error(code = ...)] should not be used with #[error(unknown)]",
        msg
    );
}

#[test]
fn it_should_throw_if_unknown_variant_has_message_attr() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {
        #[error(unknown)]
        #[error(message = "Hi")]
        Hello(N)
    }
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!(
        "#[error(message = ...)] should not be used with #[error(unknown)]",
        msg
    );
}

#[test]
fn it_should_require_unknown_variant() {
    let input = parse_input(
        r##"
    #[derive(Error)]
    pub enum That {}
    "##,
    );

    let msg = extract_error_msg(expand(&input));
    assert_eq!("#[error(unknown)] must be used", msg);
}
