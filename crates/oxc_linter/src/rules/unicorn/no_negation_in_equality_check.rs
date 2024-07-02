use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_negation_in_equality_check_diagnostic(
    span0: Span,
    suggested_operator: BinaryOperator,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-unicorn(no-negation-in-equality-check): Negated expression is not allowed in equality check.",
    )
    .with_help(format!("Remove the negation operator and use '{}' instead.", suggested_operator.as_str()))
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoNegationInEqualityCheck;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow negated expressions on the left of (in)equality checks.
    ///
    /// ### Why is this bad?
    ///
    /// A negated expression on the left of an (in)equality check is likely a mistake from trying to negate the whole condition.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    ///
    /// if (!foo === bar) {}
    ///
    /// if (!foo !== bar) {}
    ///
    /// // Good
    ///
    /// if (foo !== bar) {}
    ///
    /// if (!(foo === bar)) {}
    /// ```
    NoNegationInEqualityCheck,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

impl Rule for NoNegationInEqualityCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(binary_expr) => {
                let Expression::UnaryExpression(left_unary_expr) = &binary_expr.left else {
                    return;
                };

                if left_unary_expr.operator != UnaryOperator::LogicalNot {
                    return;
                }

                if let Expression::UnaryExpression(left_nested_unary_expr) =
                    &left_unary_expr.argument
                {
                    if left_nested_unary_expr.operator == UnaryOperator::LogicalNot {
                        return;
                    }
                }

                if !binary_expr.operator.is_equality() {
                    return;
                }

                let Some(suggested_operator) = binary_expr.operator.equality_inverse_operator()
                else {
                    return;
                };

                ctx.diagnostic(no_negation_in_equality_check_diagnostic(
                    binary_expr.span,
                    suggested_operator,
                ));
            }
            _ => {
                return;
            }
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "!foo instanceof bar",
        "+foo === bar",
        "!(foo === bar)",
        "!!foo === bar",
        "!!!foo === bar",
        "foo === !bar",
    ];

    let fail = vec![
        "!foo === bar",
        "!foo !== bar",
        "!foo == bar",
        "!foo != bar",
        "
						function x() {
							return!foo === bar;
						}
					",
        "
						function x() {
							return!
								foo === bar;
							throw!
								foo === bar;
						}
					",
        "
						foo
						!(a) === b
					",
        "
						foo
						![a, b].join('') === c
					",
        "
						foo
						! [a, b].join('') === c
					",
        "
						foo
						!/* comment */[a, b].join('') === c
					",
    ];

    Tester::new(NoNegationInEqualityCheck::NAME, pass, fail).test_and_snapshot();
}