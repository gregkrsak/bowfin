use std::fmt;

use crate::ast::{
    BinaryOp, Call, Expr, ExprKind, FieldAccess, FunctionDecl, Program, RangeType, RecordBinding,
    RecordDef, Statement, Subtract, UnaryOp, ValueFunction, ValueParameter, ValueStatement,
    VerifiedFunction,
};
use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message, self.span.line, self.span.column
        )
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: &[Token]) -> Result<Program, ParseError> {
    Parser {
        tokens,
        current: 0,
        context: None,
    }
    .parse_program()
}

pub(crate) fn parse_bounded<'a>(
    tokens: &'a [Token],
    context: &'a crate::bounded::Context,
) -> Result<Program, ParseError> {
    Parser {
        tokens,
        current: 0,
        context: Some(context),
    }
    .parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    context: Option<&'a crate::bounded::Context>,
}

impl Parser<'_> {
    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::default();

        while !self.at(&TokenKind::Eof) {
            if self.at(&TokenKind::Type) {
                program.ranges.push(self.parse_range_type()?);
            } else if self.at(&TokenKind::Record) {
                program.records.push(self.parse_record()?);
            } else if self.at(&TokenKind::Verified) {
                program
                    .functions
                    .push(FunctionDecl::Verified(self.parse_verified_function()?));
            } else if self.at(&TokenKind::Fn) {
                program
                    .functions
                    .push(FunctionDecl::Ordinary(self.parse_value_function()?));
            } else if self.at(&TokenKind::Let) {
                program
                    .statements
                    .push(Statement::Binding(self.parse_binding()?));
            } else if matches!(self.peek_kind(), TokenKind::Ident(_))
                && self.lookahead_is(1, &TokenKind::LParen)
            {
                program.statements.push(Statement::Call(self.parse_call()?));
            } else {
                return Err(self.error(format!(
                    "unsupported or unexpected top-level token in prototype: {:?}",
                    self.peek_kind()
                )));
            }
        }

        Ok(program)
    }

    fn parse_range_type(&mut self) -> Result<RangeType, ParseError> {
        let span = self.expect(&TokenKind::Type)?.span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equal)?;
        self.expect(&TokenKind::Range)?;
        let min = self.expect_number()?;
        self.expect(&TokenKind::DotDot)?;
        let max = self.expect_number()?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(RangeType {
            name,
            min,
            max,
            span,
        })
    }

    fn parse_record(&mut self) -> Result<RecordDef, ParseError> {
        let span = self.expect(&TokenKind::Record)?.span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        if self.at(&TokenKind::RBrace) {
            return Err(
                self.error("record declaration requires at least one named-range field".into())
            );
        }
        let mut fields = Vec::new();
        loop {
            let start = self.tokens[self.current].span;
            if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
                return Err(self.error("record declaration requires a field name".into()));
            }
            let name = self.expect_ident()?;
            if !self.at(&TokenKind::Colon) {
                return Err(
                    self.error("record declaration requires ':' after the field name".into())
                );
            }
            self.advance();
            let ty = self.expect_ident()?;
            fields.push(crate::ast::RecordFieldDecl {
                name,
                ty,
                span: Span {
                    end: self.tokens[self.current - 1].span.end,
                    ..start
                },
            });
            if self.at(&TokenKind::RBrace) {
                break;
            }
            if !self.at(&TokenKind::Comma) {
                return Err(self.error(
                    "record declaration requires ',' between fields and a closing brace".into(),
                ));
            }
            self.advance();
            if self.at(&TokenKind::RBrace) {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(RecordDef { name, fields, span })
    }

    fn parse_verified_function(&mut self) -> Result<VerifiedFunction, ParseError> {
        let span = self.expect(&TokenKind::Verified)?.span;
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;

        let state_param = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Amp)?;
        self.expect(&TokenKind::Mut)?;
        let state_type = self.expect_ident()?;
        self.expect(&TokenKind::Comma)?;

        let amount_param = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let amount_type = self.expect_ident()?;
        self.expect(&TokenKind::RParen)?;

        self.expect(&TokenKind::Requires)?;
        let requires = self.parse_expression(0)?;
        self.expect(&TokenKind::Ensures)?;
        let ensures = self.parse_condition()?;

        self.expect(&TokenKind::LBrace)?;
        let mut body = Vec::new();
        while !self.at(&TokenKind::RBrace) {
            let target = self.parse_field_access()?;
            let statement_span = target.span;
            self.expect(&TokenKind::MinusEqual)?;
            let operand = self.parse_expression(0)?;
            self.expect(&TokenKind::Semicolon)?;
            body.push(Subtract {
                target,
                operand,
                span: statement_span,
            });
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(VerifiedFunction {
            name,
            span,
            state_param,
            state_type,
            amount_param,
            amount_type,
            requires,
            ensures,
            body,
        })
    }

    fn parse_value_type(&mut self) -> Result<crate::ast::ValueType, ParseError> {
        let mut references = Vec::new();
        while self.at(&TokenKind::Amp) || self.at(&TokenKind::AndAnd) {
            if self.at(&TokenKind::AndAnd) {
                references.push(crate::ast::BorrowKind::Shared);
            }
            self.advance();
            let kind = if self.at(&TokenKind::Mut) {
                self.advance();
                crate::ast::BorrowKind::Mutable
            } else {
                crate::ast::BorrowKind::Shared
            };
            references.push(kind);
        }
        let mut result_nodes = Vec::new();
        let name = if self.at(&TokenKind::Result) {
            self.parse_result_type_node(&mut result_nodes, 0)?;
            "Result".into()
        } else if self.at(&TokenKind::Bool) {
            self.advance();
            "bool".into()
        } else {
            self.expect_ident()?
        };
        Ok(crate::ast::ValueType {
            name,
            references,
            result_nodes,
        })
    }
    fn parse_result_type_node(
        &mut self,
        nodes: &mut Vec<crate::ast::ResultTypeSyntax>,
        depth: usize,
    ) -> Result<usize, ParseError> {
        let limit = self.context.map_or(32, |c| c.limits.traversal_depth);
        if depth >= limit {
            return Err(
                self.result_resource_error(crate::bounded::ResourceFailure::limit(
                    "Result syntax depth",
                    self.peek().span,
                )),
            );
        }
        if nodes.len() >= 4096 {
            return Err(
                self.result_resource_error(crate::bounded::ResourceFailure::limit(
                    "Result syntax nodes",
                    self.peek().span,
                )),
            );
        }
        let node = if self.at(&TokenKind::Result) {
            self.advance();
            self.expect(&TokenKind::Less)?;
            if self.expect_ident()? != "success" {
                return Err(self.error(
                    "Result requires `success:` followed by `failure:` exactly once in that order"
                        .into(),
                ));
            }
            self.expect(&TokenKind::Colon)?;
            let success = self.parse_result_type_node(nodes, depth + 1)?;
            self.expect(&TokenKind::Comma)?;
            if self.expect_ident()? != "failure" {
                return Err(
                    self.error("Result requires `failure:` after its success payload".into())
                );
            }
            self.expect(&TokenKind::Colon)?;
            let failure = self.parse_result_type_node(nodes, depth + 1)?;
            self.expect(&TokenKind::Greater)?;
            crate::ast::ResultTypeSyntax::Result { success, failure }
        } else if self.at(&TokenKind::Bool) {
            self.advance();
            crate::ast::ResultTypeSyntax::Owned("bool".into())
        } else {
            crate::ast::ResultTypeSyntax::Owned(self.expect_ident()?)
        };
        // A parent may finish after its children fill the flat node budget.
        if nodes.len() >= 4096 {
            return Err(
                self.result_resource_error(crate::bounded::ResourceFailure::limit(
                    "Result syntax nodes",
                    self.peek().span,
                )),
            );
        }
        crate::bounded::result_admission::reserve(
            nodes,
            1,
            crate::bounded::result_admission::Allocation::Syntax,
            self.peek().span,
        )
        .map_err(|failure| self.result_resource_error(failure))?;
        let id = nodes.len();
        nodes.push(node);
        Ok(id)
    }
    fn parse_value_function(&mut self) -> Result<ValueFunction, ParseError> {
        let start = self.expect(&TokenKind::Fn)?.span;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let mut parameters = Vec::new();
        if !self.at(&TokenKind::RParen) {
            loop {
                let span = self.tokens[self.current].span;
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let ty = self.parse_value_type()?;
                let span = Span {
                    end: self.tokens[self.current - 1].span.end,
                    ..span
                };
                parameters.push(ValueParameter { name, ty, span });
                if !self.at(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(&TokenKind::RParen)?;
        let return_type = if self.at(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_value_type()?)
        } else {
            None
        };
        let effects = if matches!(self.peek_kind(), TokenKind::Ident(name) if name == "effects") {
            let span = self.peek().span;
            self.advance();
            let mut names = Vec::new();
            loop {
                if names.len() == 8 {
                    return Err(self
                        .error("an effects clause has at most eight distinct identities".into()));
                }
                let span = self.peek().span;
                let name = self.expect_ident()?;
                names.push(crate::ast::EffectName { name, span });
                if !self.at(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            let end = names.last().expect("nonempty effect clause").span.end;
            Some(crate::ast::EffectClause {
                span: Span { end, ..span },
                names,
            })
        } else {
            None
        };
        self.expect(&TokenKind::LBrace)?;
        let (body, end) = self.parse_value_block(0)?;
        Ok(ValueFunction {
            name,
            parameters,
            return_type,
            effects,
            body,
            span: Span { end, ..start },
        })
    }

    fn parse_value_block(
        &mut self,
        loop_depth: usize,
    ) -> Result<(Vec<ValueStatement>, usize), ParseError> {
        let _depth = self
            .context
            .map(|c| c.enter(self.tokens[self.current].span))
            .transpose()
            .map_err(|_| self.error("configured parser depth exhausted".into()))?;
        let mut body = Vec::new();
        while !self.at(&TokenKind::RBrace) {
            let span = self.tokens[self.current].span;
            if self.at(&TokenKind::Match) {
                self.advance();
                let scrutinee = self.parse_condition()?;
                self.expect(&TokenKind::LBrace)?;
                let mut arms: Vec<crate::ast::MatchArm> = Vec::new();
                while !self.at(&TokenKind::RBrace) {
                    let pattern_start = self.peek().span;
                    let tag = self.parse_result_tag()?;
                    if let Some(previous) = arms.iter().find(|arm| arm.tag == tag) {
                        return Err(self.error(format!("duplicate canonical {tag:?} arm; previous arm at {}:{}; Error and Err denote one failure variant", previous.pattern_span.line, previous.pattern_span.column)));
                    }
                    self.expect(&TokenKind::LParen)?;
                    let binding = self.expect_ident()?;
                    let pattern_end = self.expect(&TokenKind::RParen)?.span.end;
                    self.expect(&TokenKind::FatArrow)?;
                    self.expect(&TokenKind::LBrace)?;
                    let (body, end) = self.parse_value_block(loop_depth)?;
                    arms.push(crate::ast::MatchArm {
                        tag,
                        binding,
                        pattern_span: Span {
                            end: pattern_end,
                            ..pattern_start
                        },
                        body,
                        span: Span {
                            end,
                            ..pattern_start
                        },
                    });
                }
                if arms.len() != 2 {
                    let missing = if arms.iter().any(|a| a.tag == crate::ast::ResultTag::Ok) {
                        "failure arm (Error or Err)"
                    } else {
                        "Ok arm"
                    };
                    return Err(self.error(format!("Result match is missing its {missing}")));
                }
                let end = self.expect(&TokenKind::RBrace)?.span.end;
                body.push(ValueStatement::Match {
                    scrutinee,
                    arms,
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::Break) {
                if loop_depth == 0 {
                    return Err(
                        self.error("break is permitted only inside an ordinary while loop".into())
                    );
                }
                self.advance();
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::Break {
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::Continue) {
                if loop_depth == 0 {
                    return Err(self
                        .error("continue is permitted only inside an ordinary while loop".into()));
                }
                self.advance();
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::Continue {
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::While) {
                self.advance();
                let condition = self.parse_condition()?;
                self.expect(&TokenKind::LBrace)?;
                let (loop_body, end) = self.parse_value_block(loop_depth + 1)?;
                body.push(ValueStatement::While {
                    condition,
                    body: loop_body,
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::If) {
                self.advance();
                let condition = self.parse_condition()?;
                self.expect(&TokenKind::LBrace)?;
                let (then_body, mut end) = self.parse_value_block(loop_depth)?;
                let else_body = if self.at(&TokenKind::Else) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    let (body, branch_end) = self.parse_value_block(loop_depth)?;
                    end = branch_end;
                    Some(body)
                } else {
                    None
                };
                body.push(ValueStatement::If {
                    condition,
                    then_body,
                    else_body,
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::Let) {
                self.advance();
                let mutable = self.at(&TokenKind::Mut);
                if mutable {
                    self.advance();
                }
                let binding = self.peek().span;
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Equal)?;
                let initializer = self.parse_initializer()?;
                let propagation = if matches!(self.peek_kind(), TokenKind::Ident(name) if name == "or")
                {
                    let phrase = self.advance().span;
                    let end = self.expect(&TokenKind::Return)?.span.end;
                    Some(crate::ast::PropagationSyntax {
                        phrase: Span { end, ..phrase },
                        binding,
                    })
                } else {
                    None
                };
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::Let {
                    name,
                    mutable,
                    initializer,
                    propagation,
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::Star) {
                let reference = self.parse_deref_target()?;
                self.expect(&TokenKind::Equal)?;
                let value = self.parse_expression(0)?;
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::DerefAssign {
                    reference,
                    value,
                    span: Span { end, ..span },
                });
            } else if matches!(self.peek_kind(), TokenKind::Ident(_))
                && self
                    .tokens
                    .get(self.current + 1)
                    .is_some_and(|token| matches!(token.kind, TokenKind::Equal))
            {
                let target = self.expect_ident()?;
                self.expect(&TokenKind::Equal)?;
                let value = self.parse_expression(0)?;
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::Assign {
                    target,
                    value,
                    span: Span { end, ..span },
                });
            } else if self.at(&TokenKind::Return) {
                self.advance();
                let value = if self.at(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_expression(0)?)
                };
                if !self.at(&TokenKind::Semicolon) {
                    return Err(self.error("value-returning Bowfin functions require `return expression;` with a semicolon".into()));
                }
                let end = self.advance().span.end;
                let span = Span { end, ..span };
                body.push(match value {
                    Some(value) => ValueStatement::Return { value, span },
                    None => ValueStatement::ReturnNoValue { span },
                });
                if !self.at(&TokenKind::RBrace) {
                    return Err(self.error("return must terminate its lexical statement block; statements after return are not supported".into()));
                }
            } else if matches!(self.peek_kind(), TokenKind::Ident(_))
                && self.lookahead_is(1, &TokenKind::LParen)
            {
                // Exactly a named call, never an arbitrary expression statement.
                let call = self.parse_primary()?;
                let ExprKind::Call { callee, arguments } = call.kind else {
                    unreachable!("named call prefix")
                };
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::CallNoValue {
                    callee,
                    arguments,
                    span: Span { end, ..span },
                });
            } else if matches!(self.peek_kind(), TokenKind::Ident(_))
                && self.lookahead_is(1, &TokenKind::Dot)
            {
                let owner_span = self.peek().span;
                let owner = self.expect_ident()?;
                self.expect(&TokenKind::Dot)?;
                let field_span = self.peek().span;
                if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
                    return Err(
                        self.error("field assignment requires a field name after `.`".into())
                    );
                }
                let field = self.expect_ident()?;
                if self.at(&TokenKind::Dot) {
                    return Err(
                        self.error("temporary and nested field projection are unsupported".into())
                    );
                }
                if !self.at(&TokenKind::Equal) {
                    return Err(self.error("field assignment requires `=`; compound assignment and general field expression statements are unsupported".into()));
                }
                self.advance();
                if self.at(&TokenKind::Semicolon)
                    || self.at(&TokenKind::RBrace)
                    || self.at(&TokenKind::Eof)
                {
                    return Err(self.error("field assignment requires an RHS expression".into()));
                }
                let value = self.parse_expression(0)?;
                let end = self.expect(&TokenKind::Semicolon)?.span.end;
                body.push(ValueStatement::FieldAssign {
                    owner,
                    field,
                    owner_span,
                    field_span,
                    value,
                    span: Span { end, ..span },
                });
            } else {
                return Err(self.error("value-returning Bowfin functions require `return expression;`; bare expression statements are not supported".into()));
            }
            if !body.last().expect("parsed statement").falls_through()
                && !self.at(&TokenKind::RBrace)
            {
                let message = if matches!(body.last(), Some(ValueStatement::Continue { .. })) {
                    "statements after continue are unsupported in the current lexical block"
                } else if matches!(body.last(), Some(ValueStatement::Break { .. })) {
                    "statements after break are unsupported in the current lexical block"
                } else {
                    "statements after a conditional with no fallthrough are unsupported in the current lexical block"
                };
                return Err(self.error(message.into()));
            }
        }
        let end = self.expect(&TokenKind::RBrace)?.span.end;
        Ok((body, end))
    }

    // Conditional values are initializer-only, recursively at whole branch results.
    // They deliberately do not participate in the general expression grammar.
    fn parse_initializer(&mut self) -> Result<Expr, ParseError> {
        let _depth = self
            .context
            .map(|c| c.enter(self.tokens[self.current].span))
            .transpose()
            .map_err(|_| self.error("configured parser depth exhausted".into()))?;
        if !self.at(&TokenKind::If) {
            return self.parse_expression(0);
        }
        let span = self.advance().span;
        let condition = self.parse_condition()?;
        self.expect(&TokenKind::LBrace)?;
        let then_value = self.parse_initializer()?;
        self.expect(&TokenKind::RBrace)?;
        if !self.at(&TokenKind::Else) {
            return Err(self.error("conditional initializer requires an else branch".into()));
        }
        self.advance();
        self.expect(&TokenKind::LBrace)?;
        let else_value = self.parse_initializer()?;
        let end = self.expect(&TokenKind::RBrace)?.span.end;
        let expression = Expr {
            kind: ExprKind::IfValue {
                condition: Box::new(condition),
                then_value: Box::new(then_value),
                else_value: Box::new(else_value),
            },
            span: Span { end, ..span },
        };
        if let Some(c) = self.context {
            c.expression(&expression)
                .map_err(|_| self.error("configured expression depth exhausted".into()))?;
        }
        Ok(expression)
    }

    // Precedence climbing: only the explicitly authorized operators participate.
    fn parse_expression(&mut self, minimum: u8) -> Result<Expr, ParseError> {
        self.parse_expression_at(minimum, false)
    }
    fn parse_condition(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression_at(0, true)
    }
    fn parse_expression_at(
        &mut self,
        minimum: u8,
        block_boundary: bool,
    ) -> Result<Expr, ParseError> {
        let _depth = self
            .context
            .map(|c| c.enter(self.tokens[self.current].span))
            .transpose()
            .map_err(|_| self.error("configured parser depth exhausted".into()))?;
        let mut left = self.parse_primary_at(block_boundary)?;
        let mut comparison_seen = None;
        while let Some((op, precedence)) = self.binary_operator() {
            if precedence < minimum {
                break;
            }
            let comparison = matches!(op, BinaryOp::LessEqual | BinaryOp::Equal);
            if comparison && comparison_seen == Some(op) {
                return Err(
                    self.error("comparison operators are non-associative; use parentheses".into())
                );
            }
            self.advance();
            let right = self.parse_expression_at(precedence + 1, block_boundary)?;
            let span = Span {
                end: right.span.end,
                ..left.span
            };
            left = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            };
            if let Some(c) = self.context {
                c.expression(&left)
                    .map_err(|_| self.error("configured expression depth exhausted".into()))?;
            }
            if comparison {
                comparison_seen = Some(op);
            }
        }
        Ok(left)
    }
    fn binary_operator(&self) -> Option<(BinaryOp, u8)> {
        match self.peek_kind() {
            TokenKind::OrOr => Some((BinaryOp::Or, 1)),
            TokenKind::AndAnd => Some((BinaryOp::And, 2)),
            TokenKind::EqualEqual => Some((BinaryOp::Equal, 3)),
            TokenKind::LessEqual => Some((BinaryOp::LessEqual, 4)),
            TokenKind::Minus => Some((BinaryOp::Subtract, 5)),
            _ => None,
        }
    }
    fn parse_deref_target(&mut self) -> Result<String, ParseError> {
        self.expect(&TokenKind::Star)?;
        if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
            return Err(self.error("dereference requires a directly named ordinary reference parameter or local; arbitrary expressions and nested dereference are unsupported".into()));
        }
        let reference = self.expect_ident()?;
        if self.at(&TokenKind::LParen) || self.at(&TokenKind::Dot) {
            return Err(self.error("dereference requires a directly named reference; calls and field projection are unsupported".into()));
        }
        Ok(reference)
    }
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary_at(false)
    }
    fn parse_primary_at(&mut self, block_boundary: bool) -> Result<Expr, ParseError> {
        let _depth = self
            .context
            .map(|c| c.enter(self.tokens[self.current].span))
            .transpose()
            .map_err(|_| self.error("configured parser depth exhausted".into()))?;
        let start = self.tokens[self.current].span;
        let kind = match self.peek_kind() {
            TokenKind::Ok | TokenKind::Error => {
                let variant_span = self.peek().span;
                let tag = self.parse_result_tag()?;
                self.expect(&TokenKind::LParen)?;
                let payload = Box::new(self.parse_expression(0)?);
                self.expect(&TokenKind::RParen)?;
                ExprKind::ResultConstruct {
                    tag,
                    payload,
                    variant_span,
                }
            }
            TokenKind::True | TokenKind::False => {
                let value = self.at(&TokenKind::True);
                self.advance();
                ExprKind::BoolLiteral(value)
            }
            TokenKind::Number(_) | TokenKind::Minus => {
                ExprKind::IntegerLiteral(self.expect_number()?)
            }
            TokenKind::Star => ExprKind::Deref {
                reference: self.parse_deref_target()?,
            },
            TokenKind::Amp => {
                self.advance();
                let kind = if self.at(&TokenKind::Mut) {
                    self.advance();
                    crate::ast::BorrowKind::Mutable
                } else {
                    crate::ast::BorrowKind::Shared
                };
                let target = self.expect_ident()?;
                if self.at(&TokenKind::LParen) || self.at(&TokenKind::Dot) {
                    return Err(self.error("borrowing supports only a direct owned parameter or local name; not calls, fields, or temporaries".into()));
                }
                ExprKind::Borrow { kind, target }
            }
            TokenKind::Not => {
                self.advance();
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(self.parse_primary_at(block_boundary)?),
                }
            }
            TokenKind::Ident(_) => {
                let construction = self.lookahead_is(1, &TokenKind::LBrace)
                    && (!block_boundary
                        || self.lookahead_is(3, &TokenKind::Colon)
                        || self.lookahead_is(2, &TokenKind::Colon));
                if construction {
                    let record_span = self.tokens[self.current].span;
                    let record = self.expect_ident()?;
                    self.advance(); // brace, recognized without type-name lookup
                    if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
                        return Err(self.error(
                            "record construction requires at least one named field: initializer"
                                .into(),
                        ));
                    }
                    let mut fields = Vec::new();
                    loop {
                        let start = self.tokens[self.current].span;
                        if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
                            return Err(self.error(
                                "record construction requires a field name after ','".into(),
                            ));
                        }
                        let name = self.expect_ident()?;
                        if !self.at(&TokenKind::Colon) {
                            return Err(self.error(
                                "record construction requires ':' after the field name".into(),
                            ));
                        }
                        self.advance();
                        if self.at(&TokenKind::RBrace)
                            || self.at(&TokenKind::Semicolon)
                            || self.at(&TokenKind::Eof)
                            || self.at(&TokenKind::Comma)
                        {
                            return Err(self
                                .error("record construction requires a field initializer".into()));
                        }
                        let value = self.parse_expression(0)?;
                        fields.push(crate::ast::RecordFieldInit {
                            name,
                            span: Span {
                                end: value.span.end,
                                ..start
                            },
                            value,
                        });
                        if self.at(&TokenKind::RBrace) {
                            break;
                        }
                        if self.at(&TokenKind::Semicolon) {
                            return Err(self.error("record construction requires ',' between entries; semicolon separators are unsupported".into()));
                        }
                        if self.at(&TokenKind::Eof) {
                            return Err(
                                self.error("record construction requires a closing brace".into())
                            );
                        }
                        if !self.at(&TokenKind::Comma) {
                            return Err(self.error("record construction requires ',' between entries and a closing brace".into()));
                        }
                        self.advance();
                        if self.at(&TokenKind::RBrace) {
                            break;
                        }
                    }
                    self.advance();
                    ExprKind::RecordConstruct {
                        record,
                        fields,
                        record_span,
                    }
                } else if self.lookahead_is(1, &TokenKind::Dot) {
                    ExprKind::FieldAccess(self.parse_field_access()?)
                } else {
                    let callee = self.expect_ident()?;
                    if self.at(&TokenKind::LParen) {
                        self.advance();
                        let mut arguments = Vec::new();
                        if !self.at(&TokenKind::RParen) {
                            loop {
                                arguments.push(self.parse_expression(0)?);
                                if !self.at(&TokenKind::Comma) {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        self.expect(&TokenKind::RParen)?;
                        ExprKind::Call { callee, arguments }
                    } else {
                        ExprKind::Name(callee)
                    }
                }
            }
            TokenKind::Old => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let inner = self.parse_expression(0)?;
                self.expect(&TokenKind::RParen)?;
                let ExprKind::FieldAccess(field) = inner.kind else {
                    return Err(ParseError {
                        span: inner.span,
                        message: "old(...) supports only the mutable state field in this prototype"
                            .into(),
                    });
                };
                ExprKind::OldField(field)
            }
            TokenKind::LParen => {
                self.advance();
                let expression = self.parse_expression(0)?;
                self.expect(&TokenKind::RParen)?;
                expression.kind
            }
            other => {
                return Err(self.error(format!("unsupported or malformed expression: {other:?}")))
            }
        };
        if self.at(&TokenKind::Dot) {
            return Err(self.error("field access through a temporary or nested projection is unsupported; use a directly named ordinary record owner or reference".into()));
        }
        let expression = Expr {
            kind,
            span: Span {
                end: self.tokens[self.current - 1].span.end,
                ..start
            },
        };
        if let Some(c) = self.context {
            c.expression(&expression)
                .map_err(|_| self.error("configured expression depth exhausted".into()))?;
        }
        Ok(expression)
    }

    fn parse_binding(&mut self) -> Result<RecordBinding, ParseError> {
        let span = self.expect(&TokenKind::Let)?.span;

        let mutable = self.at(&TokenKind::Mut);
        if mutable {
            self.advance();
        }

        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equal)?;
        let record_type = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let field_name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let value = self.expect_number()?;
        self.expect(&TokenKind::RBrace)?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(RecordBinding {
            name,
            mutable,
            record_type,
            field_name,
            value,
            span,
        })
    }

    fn parse_call(&mut self) -> Result<Call, ParseError> {
        let span = self.tokens[self.current].span;
        let function = self.expect_ident()?;

        self.expect(&TokenKind::LParen)?;
        self.expect(&TokenKind::Amp)?;
        self.expect(&TokenKind::Mut)?;
        let binding = self.expect_ident()?;
        self.expect(&TokenKind::Comma)?;
        let amount = self.expect_number()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(Call {
            function,
            binding,
            amount,
            span,
        })
    }

    fn parse_field_access(&mut self) -> Result<FieldAccess, ParseError> {
        let span = self.tokens[self.current].span;
        let binding = self.expect_ident()?;
        self.expect(&TokenKind::Dot)?;
        let field_span = self.tokens[self.current].span;
        let field = self.expect_ident()?;
        let span = Span {
            end: self.tokens[self.current - 1].span.end,
            ..span
        };
        Ok(FieldAccess {
            binding,
            field,
            field_span,
            span,
        })
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek_kind().clone() {
            TokenKind::Ident(name) => {
                self.advance();
                Ok(name)
            }
            TokenKind::Result | TokenKind::Ok | TokenKind::Error | TokenKind::Match | TokenKind::ReservedResult(_) => Err(ParseError { span: self.peek().span, message: "the complete Result, Ok, Error, Err, and match spelling families are reserved and cannot be identifiers".into() }),
            other => Err(self.error(format!("expected identifier, found {other:?}"))),
        }
    }

    fn parse_result_tag(&mut self) -> Result<crate::ast::ResultTag, ParseError> {
        let tag = match self.peek_kind() {
            TokenKind::Ok => crate::ast::ResultTag::Ok,
            TokenKind::Error => crate::ast::ResultTag::Error,
            TokenKind::ReservedResult(name) => {
                let canonical = crate::lexer::reserved_result_family(name).unwrap_or("Result");
                return Err(self.error(format!(
                    "noncanonical reserved spelling `{name}`; use exact `{canonical}`"
                )));
            }
            _ => {
                return Err(self
                    .error("expected exact Ok(binding), Error(binding), or Err(binding)".into()))
            }
        };
        self.advance();
        Ok(tag)
    }

    fn expect_number(&mut self) -> Result<i64, ParseError> {
        let span = self.tokens[self.current].span;
        let negative = self.at(&TokenKind::Minus);
        if negative {
            self.advance();
        }
        match self.peek_kind().clone() {
            TokenKind::Number(value) => {
                self.advance();
                let value = if negative {
                    -(value as i128)
                } else {
                    value as i128
                };
                i64::try_from(value).map_err(|_| ParseError {
                    message: "integer literal is outside the prototype's signed i64 domain".into(),
                    span,
                })
            }
            other => Err(self.error(format!("expected integer, found {other:?}"))),
        }
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParseError> {
        if self.at(expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(format!(
                "unsupported or malformed prototype syntax: expected {expected:?}, found {:?}",
                self.peek_kind()
            )))
        }
    }

    fn at(&self, expected: &TokenKind) -> bool {
        same_variant(self.peek_kind(), expected)
    }

    fn lookahead_is(&self, offset: usize, expected: &TokenKind) -> bool {
        self.tokens
            .get(self.current + offset)
            .map(|token| same_variant(&token.kind, expected))
            .unwrap_or(false)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;
        if !matches!(&self.tokens[index].kind, TokenKind::Eof) {
            self.current += 1;
        }
        &self.tokens[index]
    }

    fn result_resource_error(&self, failure: crate::bounded::ResourceFailure) -> ParseError {
        if let Some(context) = self.context {
            context.fail(failure);
        }
        ParseError {
            message: failure.resource.into(),
            span: failure.span,
        }
    }

    fn error(&self, message: String) -> ParseError {
        let message = if let TokenKind::ReservedResult(name) = self.peek_kind() {
            format!("noncanonical reserved spelling `{name}`; use exact `{}` in its intrinsic position; the whole spelling family is unavailable as an identifier", crate::lexer::reserved_result_family(name).unwrap_or("Result"))
        } else {
            message
        };
        ParseError {
            message,
            span: self.tokens[self.current].span,
        }
    }
}

fn same_variant(left: &TokenKind, right: &TokenKind) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    const PROGRAM: &str = r#"
type Percent = range 0..100;

record Battery {
    charge: Percent
}

verified fn consume(
    battery: &mut Battery,
    amount: Percent
)
    requires amount <= battery.charge
    ensures battery.charge == old(battery.charge) - amount
{
    battery.charge -= amount;
}

let mut battery = Battery { charge: 80 };
consume(&mut battery, 90);
"#;

    #[test]
    fn parses_battery_vertical_slice() {
        let tokens = lex(PROGRAM).unwrap();
        let program = parse(&tokens).unwrap();

        assert_eq!(program.ranges[0].name, "Percent");
        assert!(matches!(
            &program.functions[0].as_verified().unwrap().requires.kind,
            ExprKind::Binary {
                op: BinaryOp::LessEqual,
                ..
            }
        ));
        assert!(
            matches!(&program.statements[0], Statement::Binding(binding) if binding.value == 80)
        );
        assert!(matches!(&program.statements[1], Statement::Call(call) if call.amount == 90));
        assert!(matches!(
            &program.functions[0].as_verified().unwrap().ensures.kind,
            ExprKind::Binary {
                op: BinaryOp::Equal,
                ..
            }
        ));
        assert_eq!(program.functions[0].as_verified().unwrap().body.len(), 1);
    }
}
