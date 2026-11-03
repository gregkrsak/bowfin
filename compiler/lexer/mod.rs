use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident(String),
    Number(u64),
    Type,
    Range,
    Record,
    Verified,
    Fn,
    Return,
    While,
    Continue,
    Break,
    If,
    Else,
    Bool,
    Result,
    Ok,
    Error,
    Match,
    ReservedResult(String),
    Less,
    Greater,
    FatArrow,
    Arrow,
    Requires,
    Ensures,
    Let,
    Mut,
    Old,
    True,
    False,
    Not,
    AndAnd,
    OrOr,
    Equal,
    EqualEqual,
    LessEqual,
    DotDot,
    MinusEqual,
    Minus,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    Semicolon,
    Amp,
    Star,
    Dot,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message, self.span.line, self.span.column
        )
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
thread_local! {
    static SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn with_scan_count<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            SCANS.with(|scans| scans.set(self.0));
        }
    }
    let _restore = Restore(SCANS.with(|scans| scans.replace(0)));
    let value = f();
    (value, SCANS.with(std::cell::Cell::get))
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    lex_with_comments(source, None).map(|(tokens, _)| tokens)
}
pub(crate) fn reserved_result_family(name: &str) -> Option<&'static str> {
    ["Result", "Ok", "Error", "Err", "match"]
        .into_iter()
        .find(|canonical| name.eq_ignore_ascii_case(canonical))
}
/// Shared lexical scan; the companion boolean never retains comment text.
pub(crate) fn lex_with_comments(
    source: &str,
    context: Option<&crate::bounded::Context>,
) -> Result<(Vec<Token>, bool), LexError> {
    #[cfg(test)]
    SCANS.with(|scans| scans.set(scans.get() + 1));
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut has_comment = false;
    let mut i = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;

    while i < bytes.len() {
        let b = bytes[i];
        if let Some(c) = context {
            let span = Span {
                line,
                column,
                start: i,
                end: i,
            };
            let emits = !(matches!(b, b' ' | b'\t' | b'\r' | b'\n')
                || b == b'/' && bytes.get(i + 1) == Some(&b'/'));
            if emits
                && tokens
                    .len()
                    .checked_add(1)
                    .map_or(true, |n| n >= c.limits.tokens)
            {
                c.fail(crate::bounded::ResourceFailure::limit(
                    "token storage",
                    span,
                ));
                return Err(LexError {
                    message: "configured token limit exhausted".into(),
                    span,
                });
            }
            if emits && tokens.try_reserve(1).is_err() {
                c.fail(crate::bounded::ResourceFailure::host(
                    "token allocation",
                    span,
                ));
                return Err(LexError {
                    message: "token allocation failed".into(),
                    span,
                });
            }
        }
        match b {
            b' ' | b'\t' | b'\r' => {
                i += 1;
                column += 1;
            }
            b'\n' => {
                i += 1;
                line += 1;
                column = 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                has_comment = true;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                    column += 1;
                }
            }
            b'0'..=b'9' => {
                let start = i;
                let start_column = column;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                    column += 1;
                }
                let text = &source[start..i];
                let value = text.parse::<u64>().map_err(|_| LexError {
                    message: if context.is_some() {
                        "integer literal is out of range".into()
                    } else {
                        format!("integer literal is out of range: {text}")
                    },
                    span: Span {
                        line,
                        column: start_column,
                        start,
                        end: i,
                    },
                })?;
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    span: Span {
                        line,
                        column: start_column,
                        start,
                        end: i,
                    },
                });
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                let start_column = column;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                    column += 1;
                }

                let text = &source[start..i];
                if let Some(c) = context {
                    if text.len() > c.limits.identifier_bytes {
                        let span = Span {
                            line,
                            column: start_column,
                            start,
                            end: i,
                        };
                        c.fail(crate::bounded::ResourceFailure::limit(
                            "identifier bytes",
                            span,
                        ));
                        return Err(LexError {
                            message: "configured identifier limit exhausted".into(),
                            span,
                        });
                    }
                }
                let kind = match text {
                    "type" => TokenKind::Type,
                    "range" => TokenKind::Range,
                    "record" => TokenKind::Record,
                    "verified" => TokenKind::Verified,
                    "fn" | "function" => TokenKind::Fn,
                    "return" => TokenKind::Return,
                    "while" => TokenKind::While,
                    "continue" => TokenKind::Continue,
                    "break" => TokenKind::Break,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "bool" => TokenKind::Bool,
                    "requires" => TokenKind::Requires,
                    "ensures" => TokenKind::Ensures,
                    "let" => TokenKind::Let,
                    "mut" | "mutable" => TokenKind::Mut,
                    "old" => TokenKind::Old,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "Result" => TokenKind::Result,
                    "Ok" => TokenKind::Ok,
                    "Error" | "Err" => TokenKind::Error,
                    "match" => TokenKind::Match,
                    _ => {
                        let mut name = String::new();
                        if let Some(c) = context {
                            if name.try_reserve_exact(text.len()).is_err() {
                                let span = Span {
                                    line,
                                    column: start_column,
                                    start,
                                    end: i,
                                };
                                c.fail(crate::bounded::ResourceFailure::host(
                                    "identifier allocation",
                                    span,
                                ));
                                return Err(LexError {
                                    message: "identifier allocation failed".into(),
                                    span,
                                });
                            }
                        }
                        name.push_str(text);
                        if reserved_result_family(text).is_some() {
                            TokenKind::ReservedResult(name)
                        } else {
                            TokenKind::Ident(name)
                        }
                    }
                };

                tokens.push(Token {
                    kind,
                    span: Span {
                        line,
                        column: start_column,
                        start,
                        end: i,
                    },
                });
            }
            b'&' if i + 1 < bytes.len() && bytes[i + 1] == b'&' => {
                push_two(&mut tokens, TokenKind::AndAnd, line, column, i);
                i += 2;
                column += 2;
            }
            b'|' if i + 1 < bytes.len() && bytes[i + 1] == b'|' => {
                push_two(&mut tokens, TokenKind::OrOr, line, column, i);
                i += 2;
                column += 2;
            }
            b'!' => push_one(
                &mut tokens,
                TokenKind::Not,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                push_two(&mut tokens, TokenKind::LessEqual, line, column, i);
                i += 2;
                column += 2;
            }
            b'<' => push_one(
                &mut tokens,
                TokenKind::Less,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'>' => push_one(
                &mut tokens,
                TokenKind::Greater,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'=' if bytes.get(i + 1) == Some(&b'>') => {
                push_two(&mut tokens, TokenKind::FatArrow, line, column, i);
                i += 2;
                column += 2;
            }
            b'.' if i + 1 < bytes.len() && bytes[i + 1] == b'.' => {
                push_two(&mut tokens, TokenKind::DotDot, line, column, i);
                i += 2;
                column += 2;
            }
            b'-' if i + 1 < bytes.len() && bytes[i + 1] == b'>' => {
                push_two(&mut tokens, TokenKind::Arrow, line, column, i);
                i += 2;
                column += 2;
            }
            b'-' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                push_two(&mut tokens, TokenKind::MinusEqual, line, column, i);
                i += 2;
                column += 2;
            }
            b'=' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                push_two(&mut tokens, TokenKind::EqualEqual, line, column, i);
                i += 2;
                column += 2;
            }
            b'=' => push_one(
                &mut tokens,
                TokenKind::Equal,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'-' => push_one(
                &mut tokens,
                TokenKind::Minus,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'{' => push_one(
                &mut tokens,
                TokenKind::LBrace,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'}' => push_one(
                &mut tokens,
                TokenKind::RBrace,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'(' => push_one(
                &mut tokens,
                TokenKind::LParen,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b')' => push_one(
                &mut tokens,
                TokenKind::RParen,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b':' => push_one(
                &mut tokens,
                TokenKind::Colon,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b',' => push_one(
                &mut tokens,
                TokenKind::Comma,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b';' => push_one(
                &mut tokens,
                TokenKind::Semicolon,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'*' => push_one(
                &mut tokens,
                TokenKind::Star,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'&' => push_one(
                &mut tokens,
                TokenKind::Amp,
                line,
                column,
                &mut i,
                &mut column,
            ),
            b'.' => push_one(
                &mut tokens,
                TokenKind::Dot,
                line,
                column,
                &mut i,
                &mut column,
            ),
            _ => {
                return Err(LexError {
                    message: format!("unexpected character `{}`", b as char),
                    span: Span {
                        line,
                        column,
                        start: i,
                        end: i + 1,
                    },
                });
            }
        }
    }

    if let Some(c) = context {
        if tokens.try_reserve(1).is_err() {
            let span = Span {
                line,
                column,
                start: i,
                end: i,
            };
            c.fail(crate::bounded::ResourceFailure::host(
                "token allocation",
                span,
            ));
            return Err(LexError {
                message: "token allocation failed".into(),
                span,
            });
        }
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            line,
            column,
            start: source.len(),
            end: source.len(),
        },
    });

    Ok((tokens, has_comment))
}

fn push_one(
    tokens: &mut Vec<Token>,
    kind: TokenKind,
    line: usize,
    column: usize,
    i: &mut usize,
    current_column: &mut usize,
) {
    let start = *i;
    tokens.push(Token {
        kind,
        span: Span {
            line,
            column,
            start,
            end: start + 1,
        },
    });
    *i += 1;
    *current_column += 1;
}

fn push_two(tokens: &mut Vec<Token>, kind: TokenKind, line: usize, column: usize, start: usize) {
    tokens.push(Token {
        kind,
        span: Span {
            line,
            column,
            start,
            end: start + 2,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_compact_and_explicit_keyword_aliases() {
        let compact = lex("fn f(x: &mut T) {}").unwrap();
        let explicit = lex("function f(x: &mutable T) {}").unwrap();

        let compact_kinds: Vec<_> = compact.into_iter().map(|token| token.kind).collect();
        let explicit_kinds: Vec<_> = explicit.into_iter().map(|token| token.kind).collect();

        assert_eq!(compact_kinds, explicit_kinds);
    }

    #[test]
    fn lexes_range_and_precondition_operators() {
        let tokens = lex("type Percent = range 0..100; requires amount <= battery.charge").unwrap();

        assert!(tokens.iter().any(|token| token.kind == TokenKind::DotDot));
        assert!(tokens
            .iter()
            .any(|token| token.kind == TokenKind::LessEqual));
    }
}
