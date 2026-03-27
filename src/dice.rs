use crate::{
    Advantage, Context, Crit, Error, Formatter, MarkdownFormatter, SimpleFormatter, ast,
    context::ExpressionContext,
    expr::{BinOp, Dice, Expression, Literal, Number, NumberKind, Parenthetical, Set, SetOp, UnOp},
};
use chumsky::Parser;
use std::sync::Arc;

#[cfg(feature = "caching")]
static CACHE: std::sync::LazyLock<quick_cache::sync::Cache<String, ast::Expression>> =
    std::sync::LazyLock::new(|| quick_cache::sync::Cache::new(256));

/// The result of a roll, containing the evaluated expression and the original
/// AST.
#[derive(Clone)]
pub struct RollResult<F: Formatter = SimpleFormatter> {
    ast: ast::Expression,
    expr: Number,
    formatter: F,
}

impl<F: Formatter> RollResult<F> {
    pub(crate) const fn new(ast: ast::Expression, roll: Number, formatter: F) -> Self {
        Self { ast, formatter, expr: roll }
    }

    /// The comment attached to the expression, if any.
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        let NumberKind::Expression(e) = &self.expr.kind() else { unreachable!() };
        e.comment.as_deref()
    }

    /// The total value of the roll (truncated).
    #[expect(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn total(&self) -> isize {
        self.expr.total() as isize
    }

    /// The actual total of the roll.
    #[must_use]
    pub fn total_f64(&self) -> f64 {
        self.expr.total()
    }

    /// Formats the roll result as a string.
    #[must_use]
    pub fn to_result(&self) -> String {
        self.formatter.format(&self.expr)
    }

    /// The parsed AST of this roll.
    #[must_use]
    pub const fn ast(&self) -> &ast::Expression {
        &self.ast
    }

    /// The crit result of this roll.
    #[must_use]
    pub fn crit(&self) -> Crit {
        let left = crate::utils::leftmost(&self.expr);
        let NumberKind::Dice(dice) = left.kind() else {
            return Crit::None;
        };
        let mut kept_set = left.kept_set();
        if !(kept_set.next().is_some() && kept_set.next().is_none() && dice.sides == 20) {
            return Crit::None;
        }
        match left.total() {
            1.0 => Crit::Failure,
            20.0 => Crit::Success,
            _ => Crit::None,
        }
    }

    /// Returns a reference to the evaluated expression of this roll.
    pub const fn expression(&self) -> &Number {
        &self.expr
    }

    /// Override the expression for this result.
    ///
    /// This does *not* sync with the AST.
    pub fn set_expression(&mut self, expr: Expression) {
        self.expr = Number::new(NumberKind::Expression(expr));
    }
}

impl<F: Formatter> core::fmt::Debug for RollResult<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RollResult").field("total", &self.total()).finish()
    }
}

impl<F: Formatter> core::fmt::Display for RollResult<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.to_result().fmt(f)
    }
}

impl<'a> From<&'a RollResult> for f64 {
    fn from(value: &'a RollResult) -> Self {
        value.expr.total()
    }
}

impl<'a> From<&'a RollResult> for isize {
    fn from(value: &'a RollResult) -> Self {
        value.total()
    }
}

impl From<RollResult> for f64 {
    fn from(value: RollResult) -> Self {
        value.expr.total()
    }
}

impl From<RollResult> for isize {
    fn from(value: RollResult) -> Self {
        value.total()
    }
}

/// A builder for [`Roller`].
#[derive(Debug)]
pub struct RollerBuilder<C: Context = ExpressionContext, F: Formatter + Clone = SimpleFormatter> {
    context: C,
    allow_comments: bool,
    advantage: Advantage,
    formatter: F,
}

impl RollerBuilder {
    const fn new() -> Self {
        Self {
            context: ExpressionContext::new(1000),
            advantage: Advantage::None,
            allow_comments: false,
            formatter: SimpleFormatter,
        }
    }
}

impl<F: Formatter + Clone, C: Context> RollerBuilder<C, F> {
    /// Sets the context for this builder.
    pub fn context<T: Context>(self, context: T) -> RollerBuilder<T, F> {
        RollerBuilder {
            context,
            allow_comments: self.allow_comments,
            formatter: self.formatter,
            advantage: self.advantage,
        }
    }

    /// Whether to allow comments while parsing.
    ///
    /// This can be overridden for individual rolls using
    /// [`Roller::roll_with_options`].
    #[must_use]
    pub const fn allow_comments(mut self, allow_comments: bool) -> Self {
        self.allow_comments = allow_comments;
        self
    }

    /// Sets the advantage for this builder.
    ///
    /// This can be overridden for individual rolls using
    /// [`Roller::roll_with_options`].
    #[must_use]
    pub const fn advantage(mut self, advantage: Advantage) -> Self {
        self.advantage = advantage;
        self
    }

    /// Sets the formatter for this builder.
    ///
    /// The formatter will be cloned for every roll.
    pub fn formatter<T: Formatter + Clone>(self, formatter: T) -> RollerBuilder<C, T> {
        RollerBuilder {
            context: self.context,
            allow_comments: self.allow_comments,
            formatter,
            advantage: self.advantage,
        }
    }

    /// Builds the [`Roller`] with the current settings.
    pub fn build(self) -> Roller<C, F> {
        Roller {
            context: self.context,
            allow_comments: self.allow_comments,
            formatter: self.formatter,
            advantage: self.advantage,
        }
    }
}

/// A roller for rolling dice expressions.
#[derive(Debug, Clone, Copy)]
pub struct Roller<C: Context = ExpressionContext, F: Formatter + Clone = SimpleFormatter> {
    context: C,
    allow_comments: bool,
    formatter: F,
    advantage: Advantage,
}

impl Roller<ExpressionContext, Arc<MarkdownFormatter>> {
    /// Creates a new [`Roller`] with a markdown formatter.
    #[inline]
    #[must_use]
    pub fn new_markdown() -> Self {
        Self {
            context: ExpressionContext::new(1000),
            allow_comments: false,
            formatter: Arc::new(MarkdownFormatter::new()),
            advantage: Advantage::None,
        }
    }
}

impl Roller {
    /// Creates a new [`Roller`] with a simple formatter.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            context: ExpressionContext::new(1000),
            allow_comments: false,
            formatter: SimpleFormatter,
            advantage: Advantage::None,
        }
    }

    /// Returns a new [`RollerBuilder`].
    #[must_use]
    pub const fn builder() -> RollerBuilder {
        RollerBuilder::new()
    }
}

impl<C: Context, F: Formatter + Clone> Roller<C, F> {
    fn eval<R: rand::Rng>(
        &self,
        node: &ast::Node,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        match node {
            ast::Node::Expression(e) => self.eval_expr(e, rng, counter),
            ast::Node::Annotated(a) => self.eval_annotated(a, rng, counter),
            ast::Node::Dice(d) => self.eval_dice(d, rng, counter),
            ast::Node::Literal(l) => Ok(Self::eval_literal(*l)),
            ast::Node::Parenthetical(p) => self.eval_parenthetical(p, rng, counter),
            ast::Node::UnOp(u) => self.eval_un_op(u, rng, counter),
            ast::Node::BinOp(b) => self.eval_bin_op(b, rng, counter),
            ast::Node::Set(s) => self.eval_set(s, rng, counter),
            ast::Node::SetExpr(s) => self.eval_set_expr(s, rng, counter),
            ast::Node::DiceExpr(d) => Self::eval_dice_expr(d, rng, counter),
        }
    }

    fn eval_expr<R: rand::Rng>(
        &self,
        node: &ast::Expression,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        Ok(Number::new(NumberKind::Expression(Expression::new(
            self.eval(&node.roll, rng, counter)?,
            node.comment.clone(),
        ))))
    }

    fn eval_annotated<R: rand::Rng>(
        &self,
        node: &ast::Annotated,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        let mut target = self.eval(&node.value, rng, counter)?;
        target.annotation = Some(node.annotations.join(""));
        Ok(target)
    }

    fn eval_literal(node: ast::Literal) -> Number {
        Number::new(NumberKind::Literal(Literal::new(node.value())))
    }

    fn eval_parenthetical<R: rand::Rng>(
        &self,
        node: &ast::Parenthetical,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        Ok(Number::new(NumberKind::Parenthetical(Parenthetical::new(self.eval(&node.value, rng, counter)?))))
    }

    fn eval_un_op<R: rand::Rng>(
        &self,
        node: &ast::UnOp,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        Ok(Number::new(NumberKind::UnOp(UnOp::new(node.op, self.eval(&node.value, rng, counter)?))))
    }

    fn eval_bin_op<R: rand::Rng>(
        &self,
        node: &ast::BinOp,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        Ok(Number::new(NumberKind::BinOp(BinOp::new(
            self.eval(&node.left, rng, counter)?,
            node.op,
            self.eval(&node.right, rng, counter)?,
        )?)))
    }

    fn eval_set<R: rand::Rng>(
        &self,
        node: &ast::Set,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        let mut target = self.eval(&node.value, rng, counter)?;
        for op in &node.ops {
            let expr_op = SetOp::from(op);
            expr_op.operate(&mut target, counter, rng)?;
            if let NumberKind::Parenthetical(p) = target.kind_mut() {
                p.ops.push(expr_op);
            } else if let NumberKind::Set(s) = target.kind_mut() {
                s.ops.push(expr_op);
            }
        }
        Ok(target)
    }

    fn eval_set_expr<R: rand::Rng>(
        &self,
        node: &ast::SetExpr,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        let values = node.values.iter().map(|v| self.eval(v, rng, counter)).collect::<Result<_, _>>()?;
        Ok(Number::new(NumberKind::Set(Set { values, ops: vec![] })))
    }

    fn eval_dice<R: rand::Rng>(
        &self,
        node: &ast::Dice,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        let mut target = self.eval(&node.value, rng, counter)?;
        for op in &node.ops {
            let expr_op = SetOp::from(op);
            expr_op.operate(&mut target, counter, rng)?;
            if let NumberKind::Parenthetical(p) = target.kind_mut() {
                p.ops.push(expr_op);
            } else if let NumberKind::Set(s) = target.kind_mut() {
                s.ops.push(expr_op);
            } else if let NumberKind::Dice(d) = target.kind_mut() {
                d.ops.push(expr_op);
            }
        }
        Ok(target)
    }

    fn eval_dice_expr<R: rand::Rng>(
        node: &ast::DiceExpr,
        rng: &mut R,
        counter: &mut C::Counter,
    ) -> Result<Number, Error<'static>> {
        Ok(Number::new(NumberKind::Dice(Dice::new(node.num, node.sides, counter, rng)?)))
    }

    #[inline]
    fn parse_comment(expr: &str) -> Result<ast::Expression, Error<'_>> {
        crate::parser::commented_expr().parse(expr).into_result().map_err(Error::Syntax)
    }

    #[inline]
    fn parse_no_comment(expr: &str) -> Result<ast::Expression, Error<'_>> {
        #[cfg(feature = "caching")]
        {
            CACHE.get_or_insert_with(expr, || crate::parser::expr().parse(expr).into_result().map_err(Error::Syntax))
        }
        #[cfg(not(feature = "caching"))]
        {
            crate::parser::expr().parse(expr).into_result().map_err(Error::Syntax)
        }
    }

    /// Parses a dice expression into an [`ast::Expression`].
    ///
    /// If the `caching` feature is enabled, and `allow_comments` is `false`,
    /// the result of parsing is stored in a global
    /// [cache][quick_cache::sync::Cache] of 256 elements.
    ///
    /// # Errors
    ///
    /// If the expression is invalid, [`Error::Syntax`] is returned.
    #[inline]
    pub fn parse<'a>(&self, expr: &'a str, allow_comments: bool) -> Result<ast::Expression, Error<'a>> {
        if allow_comments { Self::parse_comment(expr) } else { Self::parse_no_comment(expr) }
    }

    /// Rolls a parsed expression with the given advantage and RNG.
    ///
    /// # Errors
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    pub fn roll_expr<R: rand::Rng>(
        &self,
        mut expr: ast::Expression,
        advantage: Advantage,
        rng: &mut R,
    ) -> Result<RollResult<F>, Error<'static>> {
        if advantage != Advantage::None {
            let ast::Node::Expression(e) = crate::utils::ast_with_adv(expr.into(), advantage) else { unreachable!() };
            expr = e;
        }
        let mut counter = self.context.new_counter();
        let dice_expr = self.eval_expr(&expr, rng, &mut counter)?;
        self.context.finish(counter);
        Ok(RollResult::new(expr, dice_expr, self.formatter.clone()))
    }

    /// Rolls a parsed expression with the given advantage using the default
    /// RNG.
    ///
    /// # Errors
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[cfg(any(feature = "sys_rng", doc))]
    #[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "sys_rng")))]
    #[inline]
    pub fn roll_expr_with_default_rng(
        &self,
        expr: ast::Expression,
        advantage: Advantage,
    ) -> Result<RollResult<F>, Error<'static>> {
        self.roll_expr(expr, advantage, &mut crate::rng())
    }

    /// Rolls a dice expression.
    ///
    /// # Errors
    ///
    /// If parsing fails, [`Error::Syntax`] is returned.
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    #[cfg(any(feature = "sys_rng", doc))]
    #[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "sys_rng")))]
    pub fn roll<'a>(&self, expr: &'a str) -> Result<RollResult<F>, Error<'a>> {
        let expr = self.parse(expr, self.allow_comments)?;
        self.roll_expr(expr, self.advantage, &mut crate::rng())
    }

    /// Rolls a dice expression with the given RNG.
    ///
    /// # Errors
    ///
    /// If parsing fails, [`Error::Syntax`] is returned.
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    pub fn roll_with_rng<'a, R: rand::Rng>(&self, expr: &'a str, rng: &mut R) -> Result<RollResult<F>, Error<'a>> {
        let expr = self.parse(expr, self.allow_comments)?;
        self.roll_expr(expr, self.advantage, rng)
    }

    /// Rolls a dice expression allowing comments.
    ///
    /// # Errors
    ///
    /// If parsing fails, [`Error::Syntax`] is returned.
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    #[cfg(any(feature = "sys_rng", doc))]
    #[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "sys_rng")))]
    pub fn roll_with_comments<'a>(&self, expr: &'a str) -> Result<RollResult<F>, Error<'a>> {
        let expr = self.parse(expr, true)?;
        self.roll_expr(expr, self.advantage, &mut crate::rng())
    }

    /// Rolls a dice expression with options overriding those set in this
    /// [`Roller`].
    ///
    /// # Errors
    ///
    /// If parsing fails, [`Error::Syntax`] is returned.
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    pub fn roll_with_options<'a, R: rand::Rng>(
        &self,
        expr: &'a str,
        allow_comments: bool,
        advantage: Advantage,
        rng: &mut R,
    ) -> Result<RollResult<F>, Error<'a>> {
        let expr = self.parse(expr, allow_comments)?;
        self.roll_expr(expr, advantage, rng)
    }

    /// Rolls a dice expression with the given advantage and default RNG.
    ///
    /// # Errors
    ///
    /// If parsing fails, [`Error::Syntax`] is returned.
    ///
    /// If the expression is invalid, such as rolling a 0-sided die,
    /// [`Error::Value`] is returned. If the right hand side of a division
    /// (or modulo) is 0, [`Error::ZeroDivision`] is returned.
    #[inline]
    #[cfg(any(feature = "sys_rng", doc))]
    #[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "sys_rng")))]
    pub fn roll_with_advantage<'a>(&self, expr: &'a str, advantage: Advantage) -> Result<RollResult<F>, Error<'a>> {
        let expr = self.parse(expr, self.allow_comments)?;
        self.roll_expr(expr, advantage, &mut crate::rng())
    }
}

impl Default for Roller {
    fn default() -> Self {
        Self::new()
    }
}
