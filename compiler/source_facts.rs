//! Fixed, source-derived CLI companion facts. These are neither semantic state
//! nor execution approval, and carry no source text, path, comment or policy.
use crate::{ast, lexer::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceFacts {
    has_comment: bool,
    first_code_token: Option<Span>,
    parsed_nonempty: bool,
}

impl SourceFacts {
    /// Construct only alongside a successful parse of this exact token stream.
    pub(crate) fn from_parsed(
        has_comment: bool,
        first_code_token: Option<Span>,
        ast: &ast::Program,
    ) -> Self {
        Self {
            has_comment,
            first_code_token,
            parsed_nonempty: !ast.ranges.is_empty()
                || !ast.records.is_empty()
                || !ast.functions.is_empty()
                || !ast.statements.is_empty(),
        }
    }

    pub fn has_comment(&self) -> bool {
        self.has_comment
    }

    pub fn first_code_token(&self) -> Option<Span> {
        self.first_code_token
    }

    pub fn parsed_nonempty(&self) -> bool {
        self.parsed_nonempty
    }
}

/// Additive CLI-support pairing; construction is confined to one preparation.
/// It does not impose lint policy or a new general-purpose semantic contract.
#[derive(Debug)]
pub struct SourcePrepared<T> {
    value: T,
    facts: SourceFacts,
}

impl<T> SourcePrepared<T> {
    pub(crate) fn new(value: T, facts: SourceFacts) -> Self {
        Self { value, facts }
    }

    pub fn facts(&self) -> &SourceFacts {
        &self.facts
    }

    pub fn into_parts(self) -> (T, SourceFacts) {
        (self.value, self.facts)
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bounded, compile_source, compile_source_with_facts, lexer};
    use crate::{parse_source, parse_source_with_facts, FrontendError};

    const RANGE: &str = "type Percent = range 0..100;";

    #[test]
    fn parsed_eligibility_uses_each_source_collection() {
        for source in [
            RANGE,
            "record Battery { charge: Percent }",
            "function demo() -> bool { return true; }",
            include_str!("../examples/battery.bow"),
            "let mutable battery = Battery { charge: 80 };",
            "consume(&mutable battery, 50);",
        ] {
            let prepared = parse_source_with_facts(source).unwrap();
            assert!(prepared.facts().parsed_nonempty(), "{source}");
            assert!(prepared.facts().first_code_token().is_some(), "{source}");
        }
    }

    #[test]
    fn empty_whitespace_and_comment_only_are_ineligible() {
        for source in ["", " \t\r\n", "//", "// comment\n\t// trailing"] {
            let prepared = parse_source_with_facts(source).unwrap();
            assert!(!prepared.facts().parsed_nonempty());
            assert_eq!(prepared.facts().first_code_token(), None);
        }
    }

    #[test]
    fn every_recognized_comment_form_sets_only_presence() {
        for source in [
            format!("// purpose\n{RANGE}"),
            format!("{RANGE}// trailing"),
            "type // inline\nPercent = range 0..100;".into(),
            format!("{RANGE}//"),
            format!("{RANGE}// \t"),
            format!("// Copyright example\n{RANGE}"),
            format!("/// ordinary\n{RANGE}"),
            format!("//! ordinary\n{RANGE}"),
        ] {
            let prepared = compile_source_with_facts(&source).unwrap();
            assert!(prepared.facts().has_comment(), "{source}");
            assert!(prepared.facts().parsed_nonempty());
        }
        let prepared = compile_source_with_facts(RANGE).unwrap();
        assert!(!prepared.facts().has_comment());
    }

    #[test]
    fn first_code_span_reuses_lexer_whitespace_and_alias_coordinates() {
        for (source, line, column, start, end) in [
            ("\n\n  type P = range 0..1;", 3, 3, 4, 8),
            ("\r\n\t type P = range 0..1;", 2, 3, 4, 8),
            ("\r\tfn demo() {}", 1, 3, 2, 4),
            ("\n function demo() {}", 2, 2, 2, 10),
        ] {
            let prepared = parse_source_with_facts(source).unwrap();
            let expected = Span {
                line,
                column,
                start,
                end,
            };
            assert_eq!(prepared.facts().first_code_token(), Some(expected));
            assert_eq!(lexer::lex(source).unwrap()[0].span, expected);
        }
    }

    #[test]
    fn legacy_and_companion_ast_hir_and_failures_agree() {
        for source in [
            "",
            RANGE,
            "function demo() -> bool { return true; }",
            include_str!("../examples/battery_ok.bow"),
        ] {
            assert_eq!(
                parse_source(source).unwrap(),
                parse_source_with_facts(source).unwrap().into_value()
            );
            assert_eq!(
                compile_source(source).unwrap(),
                compile_source_with_facts(source).unwrap().into_value()
            );
        }
        for source in [
            "@ // later comment",
            "// earlier comment\n@",
            "/* unsupported */",
            "type P = range 0..1",
            "type P = range 0..1; type P = range 0..1;",
            "function demo() -> bool { return 1; }",
            "function demo() -> bool { return unknown; }",
        ] {
            assert_eq!(
                compile_source(source).unwrap_err().to_string(),
                compile_source_with_facts(source).unwrap_err().to_string(),
                "{source}"
            );
        }
    }

    #[test]
    fn malformed_input_returns_primary_error_without_successful_facts() {
        assert!(matches!(
            parse_source_with_facts("@ // observed only after invalid input"),
            Err(FrontendError::Lex(_))
        ));
        assert!(matches!(
            parse_source_with_facts("// observed\nrecord"),
            Err(FrontendError::Parse(_))
        ));
        assert!(matches!(
            compile_source_with_facts("type P = range 0..1; type P = range 0..1;"),
            Err(FrontendError::Resolve(_))
        ));
    }

    #[test]
    fn companions_each_invoke_one_shared_lexical_scan() {
        let (parsed, scans) = lexer::with_scan_count(|| parse_source_with_facts(RANGE));
        assert!(parsed.is_ok());
        assert_eq!(scans, 1);
        let (compiled, scans) = lexer::with_scan_count(|| compile_source_with_facts(RANGE));
        assert!(compiled.is_ok());
        assert_eq!(scans, 1);
        let (legacy, scans) = lexer::with_scan_count(|| compile_source(RANGE));
        assert!(legacy.is_ok());
        assert_eq!(scans, 1);
    }

    #[test]
    fn ordinary_and_bounded_scans_keep_identical_tokens_and_facts() {
        for source in [RANGE, "// header\r\n\ttype P = range 0..1;// EOF", ""] {
            let ordinary = lexer::lex_with_comments(source, None).unwrap();
            let context = bounded::Context::new(bounded::AdmissionLimits::default());
            let bounded = lexer::lex_with_comments(source, Some(&context)).unwrap();
            assert_eq!(ordinary, bounded);
            assert_eq!(ordinary.0, lexer::lex(source).unwrap());
        }
    }

    #[test]
    fn companion_metadata_does_not_change_lexical_limit_neighbors() {
        let count = lexer::lex(RANGE).unwrap().len();
        let sufficient = bounded::AdmissionLimits {
            tokens: count,
            ..bounded::AdmissionLimits::default()
        };
        let context = bounded::Context::new(sufficient);
        assert!(lexer::lex_with_comments(RANGE, Some(&context)).is_ok());
        let context = bounded::Context::new(bounded::AdmissionLimits {
            tokens: count - 1,
            ..sufficient
        });
        assert!(lexer::lex_with_comments(RANGE, Some(&context)).is_err());
        assert!(matches!(
            context.failure.get(),
            Some(bounded::ResourceFailure {
                kind: bounded::ResourceKind::Limit,
                ..
            })
        ));
    }

    #[test]
    fn returned_companion_owns_facts_after_source_storage_is_dropped() {
        let prepared = {
            let source = String::from("// purpose\nfunction demo() -> bool { return true; }");
            compile_source_with_facts(&source).unwrap()
        };
        assert!(prepared.facts().has_comment());
        let (program, facts) = prepared.into_parts();
        assert_eq!(program.functions().len(), 1);
        assert_eq!(facts.first_code_token().unwrap().line, 2);
        assert!(facts.parsed_nonempty());
    }
}
