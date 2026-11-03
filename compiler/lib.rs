pub mod ast;
#[cfg(test)]
mod bed037_tests;
mod bounded;
pub mod diagnostics;
pub mod effects;
pub mod execution;
pub mod hir;
pub mod lexer;
#[cfg(test)]
mod lint_reporting;
pub mod mir;
pub mod ownership;
pub mod parser;
pub mod resolve;
#[cfg(test)]
mod run_reporting;
pub mod source_facts;
pub mod trusted;
pub mod types;
pub mod verify;

use std::fmt;

use ast::Program;
use source_facts::{SourceFacts, SourcePrepared};

#[derive(Debug)]
pub enum FrontendError {
    Lex(lexer::LexError),
    Parse(parser::ParseError),
    Resolve(Vec<diagnostics::Diagnostic>),
    Type(Vec<diagnostics::Diagnostic>),
    Ownership(Vec<diagnostics::Diagnostic>),
    Effects(Vec<diagnostics::Diagnostic>),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
            Self::Resolve(errors)
            | Self::Type(errors)
            | Self::Ownership(errors)
            | Self::Effects(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    write!(
                        f,
                        "{} at {}:{}",
                        error.message, error.span.line, error.span.column
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for FrontendError {}

/// Parse source syntax without resolving declaration identity.
pub fn parse_source(source: &str) -> Result<Program, FrontendError> {
    parse_source_with_facts(source).map(SourcePrepared::into_value)
}

/// CLI-support companion obtained from the same successful lexical/parser pass.
/// This carries no lint policy and never prints.
pub fn parse_source_with_facts(source: &str) -> Result<SourcePrepared<Program>, FrontendError> {
    let (tokens, has_comment) =
        lexer::lex_with_comments(source, None).map_err(FrontendError::Lex)?;
    let first = tokens
        .first()
        .filter(|token| token.kind != lexer::TokenKind::Eof)
        .map(|token| token.span);
    let ast = parser::parse(&tokens).map_err(FrontendError::Parse)?;
    let facts = SourceFacts::from_parsed(has_comment, first, &ast);
    Ok(SourcePrepared::new(ast, facts))
}

/// Parse, resolve, type-check, and ownership/loan/borrowed-access-check the supported subset to canonical HIR.
/// Numerical proof and ordered call-state checking are separate verifier work.
pub fn compile_source(source: &str) -> Result<hir::Program, FrontendError> {
    compile_source_with_facts(source).map(SourcePrepared::into_value)
}

/// CLI-support compilation companion; facts remain outside canonical HIR.
pub fn compile_source_with_facts(
    source: &str,
) -> Result<SourcePrepared<hir::Program>, FrontendError> {
    let (ast, facts) = parse_source_with_facts(source)?.into_parts();
    let resolved = resolve::resolve(&ast).map_err(FrontendError::Resolve)?;
    let program = types::check(resolved).map_err(FrontendError::Type)?;
    let cleanup_sites = ownership::analyze(&program).map_err(FrontendError::Ownership)?;
    effects::calls::check(&program).map_err(FrontendError::Effects)?;
    effects::custody::check(&program, &cleanup_sites).map_err(FrontendError::Effects)?;
    Ok(SourcePrepared::new(program, facts))
}

#[cfg(test)]
mod propagation_tests;
