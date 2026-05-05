use lexer::Token;

use crate::{
    IdentifierExt, Lexer, Parser, Result,
    ast::{
        Atom, BinaryOperator, BindingPower, Expr, ExprNode, Getline, PlaceOperator, Ternary,
        UnaryOperator, Variable, WriteKind,
    },
    diagnostics::ParsingError,
    lex::TokenExt,
};

pub struct Pratt<'a, 'b> {
    parser: &'b mut Parser<'a>,
}

impl<'a, 'b> Pratt<'a, 'b> {
    pub fn parse(parser: &'b mut Parser<'a>, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        Self { parser }.parse_expression(lex, 0)
    }

    fn parse_lhs(&mut self, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        if lex.consume(&Token::OpenParent) {
            self.parse_parenthesized(lex)
        } else if lex.peek_with(Token::is_prefix_op) {
            self.parse_prefix(lex)
        } else if lex.consume(&Token::Getline) {
            self.parse_prefix_getline(lex)
        } else {
            self.parse_atom_or_call(lex)
        }
    }

    fn parse_expression(&mut self, lex: &mut Lexer<'a>, min_bp: u8) -> Result<Expr<'a>> {
        let lhs = self.parse_lhs(lex)?;
        self.fold_rhs(lex, lhs, min_bp)
    }

    fn fold_rhs(&mut self, lex: &mut Lexer<'a>, mut lhs: Expr<'a>, min_bp: u8) -> Result<Expr<'a>> {
        while let Some((next, span)) = lex.peek_with_span() {
            let next = next?;
            lhs = if let Ok(op) = BinaryOperator::parse(next, &span)
                && !matches!(next, Token::Increment | Token::Decrement)
            {
                if op.binding_power().0 < min_bp {
                    break;
                }
                self.parse_infix_op(lex, op, lhs)?
            } else if let Ok(op) = PlaceOperator::parse(next, &span) {
                let Expr::Leaf(Atom::Variable(var)) = lhs.take() else {
                    return Err(ParsingError::OperatorExpectsVariable(lex.span()));
                };
                if op.binding_power().0 < min_bp {
                    break;
                }
                self.parse_place_op(lex, op, var)?
            } else if next == &Token::QuestionMark {
                if Ternary.binding_power().0 < min_bp {
                    break;
                }
                self.parse_ternary(lex, lhs)?
            } else if let Some((op, reciprocal, bp)) = BinaryOperator::unfold_suffix(next) {
                let Expr::Leaf(Atom::Variable(rhs)) = lhs else {
                    return Err(ParsingError::OperatorExpectsVariable(lex.span()));
                };
                if bp < min_bp {
                    break;
                }
                self.unfolded_suffix_op(lex, op, reciprocal, rhs)
            } else if let Some(op) = WriteKind::parse(next) {
                self.parse_getline_pipe(lex, op, lhs)?
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_parenthesized(&mut self, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        let inner = self.parse_expression(lex, 0);
        lex.expect(
            &Token::ClosedParent,
            ParsingError::UnclosedParenthesisExpression,
        )
        .and(inner)
    }

    fn parse_prefix(&mut self, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        let next = lex.expect_next()?;
        if let Some((op, bp)) = BinaryOperator::unfold_prefix(&next) {
            let Expr::Leaf(Atom::Variable(rhs)) = self.parse_expression(lex, bp)? else {
                return Err(ParsingError::OperatorExpectsVariable(lex.span()));
            };
            Ok(Expr::node(
                PlaceOperator::Assignment.expr(
                    rhs,
                    Expr::node(op.expr(Expr::leaf(rhs), Expr::leaf(1.)), self.parser.arena),
                ),
                self.parser.arena,
            ))
        } else if let Ok(op) = UnaryOperator::parse(&next, &lex.peeked_span()?) {
            let rhs = self.parse_expression(lex, op.binding_power())?;
            Ok(Expr::node(op.expr(rhs), self.parser.arena))
        } else {
            Err(ParsingError::InvalidExpression(lex.span()))
        }
    }

    fn parse_prefix_getline(&mut self, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        // Consumes with maximum precedence the following ident and/or
        // redirection reading from file.
        // TODO: move to specialized rhs loop to disambiguate call/variable.
        let var = if lex.peek_with(Token::is_place) {
            let next = lex.expect_next()?;
            self.parser.get_place(lex, next)
        } else {
            None
        };
        if lex.consume(&Token::LesserThan) {
            Ok(Expr::node(
                Getline::FromFile(var, self.parse_expression(lex, 0)?),
                self.parser.arena,
            ))
        } else {
            Ok(Expr::node(Getline::FromInput(var), self.parser.arena))
        }
    }

    fn parse_atom_or_call(&mut self, lex: &mut Lexer<'a>) -> Result<Expr<'a>> {
        let next = lex.expect_next()?;
        if let Token::Identifier(name) = next
            && lex.peek_is(&Token::OpenParent)
            && lex.is_yuxtaposed()
        {
            self.parser
                .parse_function_call(lex, name.qualify(self.parser.namespace), lex.span())
        } else {
            Ok(Expr::leaf(self.parser.parse_atom(lex, next)?))
        }
    }

    fn parse_infix_op(
        &mut self,
        lex: &mut Lexer<'a>,
        op: BinaryOperator,
        lhs: Expr<'a>,
    ) -> Result<Expr<'a>> {
        lex.consume_with(|_| op != BinaryOperator::Concat);

        let rhs = self.parse_expression(lex, op.binding_power().1)?;
        Ok(Expr::node(op.expr(lhs, rhs), self.parser.arena))
    }

    fn parse_place_op(
        &mut self,
        lex: &mut Lexer<'a>,
        op: PlaceOperator,
        var: Variable<'a>,
    ) -> Result<Expr<'a>> {
        let token_op = lex.expect_next()?;

        let mut rhs = self.parse_expression(lex, op.binding_power().1)?;
        if let Some(op) = BinaryOperator::unfold(&token_op) {
            rhs = Expr::node(op.expr(Expr::leaf(var), rhs), self.parser.arena);
        } else if op == PlaceOperator::ArrayAccess {
            while lex.consume(&Token::Comma) {
                rhs = Expr::node(
                    BinaryOperator::Concat.expr(
                        rhs,
                        Expr::node(
                            BinaryOperator::Concat
                                .expr(Expr::leaf(Variable::Subsep), self.parse_expression(lex, 0)?),
                            self.parser.arena,
                        ),
                    ),
                    self.parser.arena,
                );
            }
            lex.expect(&Token::ClosedBracket, ParsingError::UnclosedArrayAccess)?;
        }
        Ok(Expr::node(op.expr(var, rhs), self.parser.arena))
    }

    fn parse_ternary(&mut self, lex: &mut Lexer<'a>, lhs: Expr<'a>) -> Result<Expr<'a>> {
        let right_bp = Ternary.binding_power().1;
        lex.next();
        let then_branch = self.parse_expression(lex, right_bp)?;
        lex.expect(&Token::Colon, ParsingError::MissingTernaryOr)?;
        let else_branch = self.parse_expression(lex, right_bp)?;
        Ok(Expr::node(
            ExprNode::Ternary(lhs, then_branch, else_branch),
            self.parser.arena,
        ))
    }

    fn unfolded_suffix_op(
        &mut self,
        lex: &mut Lexer<'a>,
        op: BinaryOperator,
        reciprocal: BinaryOperator,
        rhs: Variable<'a>,
    ) -> Expr<'a> {
        lex.next();
        Expr::node(
            reciprocal.expr(
                Expr::node(
                    PlaceOperator::Assignment.expr(
                        rhs,
                        Expr::node(op.expr(Expr::leaf(rhs), Expr::leaf(1.)), self.parser.arena),
                    ),
                    self.parser.arena,
                ),
                Expr::leaf(1.),
            ),
            self.parser.arena,
        )
    }

    fn parse_getline_pipe(
        &mut self,
        lex: &mut Lexer<'a>,
        op: WriteKind,
        lhs: Expr<'a>,
    ) -> Result<Expr<'a>> {
        lex.expect(&Token::Getline, |span| {
            ParsingError::UnexpectedToken(
                span,
                "operand must precede `getline` in an expression.".into(),
            )
        })?;
        // TODO: move to specialized rhs loop to disambiguate call/variable.
        Ok(Expr::node(op.expr_getline(None, lhs), self.parser.arena))
    }
}
