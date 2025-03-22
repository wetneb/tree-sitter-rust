//! This crate provides Rust language support for the [tree-sitter][] parsing library.
//!
//! Typically, you will use the [LANGUAGE][] constant to add this language to a
//! tree-sitter [Parser][], and then use the parser to parse some code:
//!
//! ```
//! use tree_sitter::Parser;
//!
//! let code = r#"
//! fn double(x: i32) -> i32 {
//!     x * 2
//! }
//! "#;
//! let mut parser = Parser::new();
//! let language = tree_sitter_rust::LANGUAGE;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading Rust parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [Parser]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_rust() -> *const ();
}

/// The tree-sitter [`LanguageFn`][LanguageFn] for this grammar.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_rust) };

/// The content of the [`node-types.json`][] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

/// The syntax highlighting query for this language.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

/// The injections query for this language.
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");

/// The symbol tagging query for this language.
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    use std::{fs::{self, File}, io::{stdout, Read, Write}};

    use walkdir::WalkDir;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Rust parser");
    }

    #[test]
    fn parse_rustc_suite() {
        let mut parser: tree_sitter::Parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Rust parser");

        let mut count = 0;
        let mut failures = Vec::new();
        for entry in WalkDir::new("/home/user/projects/tree-sitter-rust/rustc_comparison")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.file_name().to_str().is_some_and(|s| s.ends_with(".rs"))) {
            let fname = entry.path().display().to_string();
            count += 1;
            let tree_sitter_accepts = accepts_file(&fname, &mut parser);
            let syn_accepts = fname.contains("/valid/");
            if tree_sitter_accepts != syn_accepts {
                failures.push((fname, syn_accepts));
            }
        }

        failures.sort();
        for (failure, expected_to_parse) in failures.iter() {
            println!("{expected_to_parse}: {failure}");
        }
        println!("{} parse failures out of {}", failures.len(), count);
        assert_eq!(failures.len(), 0);
    }

    fn accepts_file(filename: &str, parser: &mut tree_sitter::Parser) -> bool {
        let mut f = File::open(filename).expect("no file found");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("couldn't read the source file");
        match parser.parse(&bytes, None) {
            Some(tree) => !tree.root_node().has_error(),
            None => false,
        }
    }
}
