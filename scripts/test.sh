# cargo-nextest has a problem when dealing with proc-macro crates
# especially to capwat-macros crate.
cargo nextest run -E 'not package(capwat-macros)'
