//! Types for the evaluated expression tree generated from the parsed AST.

use crate::{
    Error,
    context::RollCounter,
    utils::{float_eq, float_ne},
};
use core::fmt::Debug;
use std::{borrow::Cow, collections::HashSet};

static NAIVE_GENERATOR: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

/// A number in the expression tree.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Number {
    hash_key: usize,
    kind: NumberKind,
    kept: bool,
    pub(crate) annotation: Option<String>,
}

impl core::fmt::Debug for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Number")
            .field("kind", &self.kind)
            .field("kept", &self.kept)
            .field("annotation", &self.annotation)
            .finish_non_exhaustive()
    }
}

impl Number {
    /// Creates a new number.
    pub fn new(kind: NumberKind) -> Self {
        Self {
            hash_key: NAIVE_GENERATOR.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
            kind,
            kept: true,
            annotation: None,
        }
    }

    /// The inner kind of this number.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> &NumberKind {
        &self.kind
    }

    /// Whether this number is kept.
    #[must_use]
    #[inline]
    pub const fn kept(&self) -> bool {
        self.kept
    }

    /// The annotation of this number, if any.
    #[must_use]
    #[inline]
    pub fn annotation(&self) -> Option<&str> {
        self.annotation.as_deref()
    }

    /// The set of numbers that make up this number.
    #[must_use]
    #[inline]
    pub fn set(&self) -> Box<dyn Iterator<Item = &Self> + '_> {
        match &self.kind {
            NumberKind::Expression(e) => e.set(),
            NumberKind::Parenthetical(p) => p.set(),
            NumberKind::Set(s) => Box::new(s.set()),
            NumberKind::Dice(d) => Box::new(d.set()),
            NumberKind::Die(d) => Box::new(d.set()),
            NumberKind::UnOp(_) | NumberKind::Literal(_) | NumberKind::BinOp(_) => Box::new(core::iter::once(self)),
        }
    }

    /// The set of numbers that contribute to this number's value.
    #[inline]
    pub fn kept_set(&self) -> impl Iterator<Item = &Self> {
        self.set().filter(|n| n.kept)
    }

    /// The set of numbers that contribute to this number's value.
    #[inline]
    pub fn kept_set_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        self.set_mut().filter(|n| n.kept)
    }

    /// The set of numbers that make up this number.
    #[inline]
    pub fn set_mut(&mut self) -> Box<dyn Iterator<Item = &mut Self> + '_> {
        if let NumberKind::UnOp(_) | NumberKind::Literal(_) | NumberKind::BinOp(_) = self.kind {
            return Box::new(core::iter::once(self));
        }
        match &mut self.kind {
            NumberKind::Expression(e) => e.set_mut(),
            NumberKind::Parenthetical(p) => p.set_mut(),
            NumberKind::Set(s) => Box::new(s.set_mut()),
            NumberKind::Dice(d) => Box::new(d.set_mut()),
            NumberKind::Die(d) => Box::new(d.set_mut()),
            NumberKind::UnOp(_) | NumberKind::Literal(_) | NumberKind::BinOp(_) => unreachable!(),
        }
    }

    /// Marks this number as dropped.
    #[inline]
    pub const fn drop_(&mut self) {
        self.kept = false;
    }

    /// The value of this number.
    #[inline]
    pub fn number(&self) -> f64 {
        match &self.kind {
            NumberKind::Literal(l) => l.number(),
            NumberKind::UnOp(u) => u.number(),
            NumberKind::Expression(e) => e.number(),
            NumberKind::BinOp(b) => b.number(),
            NumberKind::Parenthetical(p) => p.number(),
            NumberKind::Die(d) => d.number(),
            _ => self.kept_set().map(Self::number).sum(),
        }
    }

    /// The value of this number, if it is kept, else 0.
    #[must_use]
    #[inline]
    pub fn total(&self) -> f64 {
        if self.kept { self.number() } else { 0.0 }
    }

    /// Returns a mutable reference to the inner kind of this number.
    #[inline]
    pub const fn kind_mut(&mut self) -> &mut NumberKind {
        &mut self.kind
    }
}

impl crate::ast::HasChildren for Number {
    type Item = Self;
    type Iter<'a> = Box<dyn Iterator<Item = &'a Self> + 'a>;
    type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Self> + 'a>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        match self.kind() {
            NumberKind::Expression(e) => Box::new(core::iter::once(&*e.roll)),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => Box::new(core::iter::empty()),
            NumberKind::UnOp(u) => Box::new(core::iter::once(&*u.value)),
            NumberKind::BinOp(b) => Box::new([&*b.left, &*b.right].into_iter()),
            NumberKind::Parenthetical(p) => Box::new(core::iter::once(&*p.value)),
            NumberKind::Set(s) => Box::new(s.set()),
        }
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        match self.kind_mut() {
            NumberKind::Expression(e) => Box::new(core::iter::once(&mut *e.roll)),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => Box::new(core::iter::empty()),
            NumberKind::UnOp(u) => Box::new(core::iter::once(&mut *u.value)),
            NumberKind::BinOp(b) => Box::new([&mut *b.left, &mut *b.right].into_iter()),
            NumberKind::Parenthetical(p) => Box::new(core::iter::once(&mut *p.value)),
            NumberKind::Set(s) => Box::new(s.set_mut()),
        }
    }

    #[inline]
    fn left(&self) -> Option<&Self::Item> {
        match self.kind() {
            NumberKind::Expression(e) => Some(&*e.roll),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => None,
            NumberKind::UnOp(u) => Some(&*u.value),
            NumberKind::BinOp(b) => Some(&*b.left),
            NumberKind::Parenthetical(p) => Some(&*p.value),
            NumberKind::Set(s) => s.values.first(),
        }
    }

    #[inline]
    fn right(&self) -> Option<&Self::Item> {
        match self.kind() {
            NumberKind::Expression(e) => Some(&*e.roll),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => None,
            NumberKind::UnOp(u) => Some(&*u.value),
            NumberKind::BinOp(b) => Some(&*b.right),
            NumberKind::Parenthetical(p) => Some(&*p.value),
            NumberKind::Set(s) => s.values.last(),
        }
    }

    #[inline]
    fn set_left<T: Into<Self::Item>>(&mut self, child: T) {
        match self.kind_mut() {
            NumberKind::Expression(e) => *e.roll = child.into(),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => {}
            NumberKind::UnOp(u) => *u.value = child.into(),
            NumberKind::BinOp(b) => *b.left = child.into(),
            NumberKind::Parenthetical(p) => *p.value = child.into(),
            NumberKind::Set(s) => s.values[0] = child.into(),
        }
    }

    #[inline]
    fn set_right<T: Into<Self::Item>>(&mut self, child: T) {
        match self.kind_mut() {
            NumberKind::Expression(e) => *e.roll = child.into(),
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => {}
            NumberKind::UnOp(u) => *u.value = child.into(),
            NumberKind::BinOp(b) => *b.right = child.into(),
            NumberKind::Parenthetical(p) => *p.value = child.into(),
            NumberKind::Set(s) => {
                let idx = s.values.len() - 1;
                s.values[idx] = child.into();
            }
        }
    }

    #[inline]
    fn set_child<T: Into<Self::Item>>(&mut self, idx: usize, child: T) {
        match self.kind_mut() {
            NumberKind::Expression(e) => {
                assert_eq!(idx, 0, "index out of bounds");
                *e.roll = child.into();
            }
            NumberKind::Literal(_) | NumberKind::Die(_) | NumberKind::Dice(_) => {}
            NumberKind::UnOp(u) => {
                assert_eq!(idx, 0, "index out of bounds");
                *u.value = child.into();
            }
            NumberKind::BinOp(b) => {
                assert!(idx < 2, "index out of bounds");
                match idx {
                    0 => *b.left = child.into(),
                    1 => *b.right = child.into(),
                    _ => unreachable!(),
                }
            }
            NumberKind::Parenthetical(p) => {
                assert_eq!(idx, 0, "index out of bounds");
                *p.value = child.into();
            }
            NumberKind::Set(s) => {
                s.values[idx] = child.into();
            }
        }
    }
}

/// The kind of number in the expression tree.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NumberKind {
    /// A roll expression.
    ///
    /// This is always the root of the expression tree.
    Expression(Expression),
    /// A literal value.
    Literal(Literal),
    /// A unary operation.
    UnOp(UnOp),
    /// A binary operation.
    BinOp(BinOp),
    /// A parenthetical value.
    Parenthetical(Parenthetical),
    /// A set of numbers.
    Set(Set),
    /// A single die.
    Die(Die),
    /// A set of dice.
    Dice(Dice),
}

impl core::fmt::Debug for NumberKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Expression(e) => e.fmt(f),
            Self::Literal(l) => l.fmt(f),
            Self::UnOp(u) => u.fmt(f),
            Self::BinOp(b) => b.fmt(f),
            Self::Parenthetical(p) => p.fmt(f),
            Self::Set(s) => s.fmt(f),
            Self::Die(d) => d.fmt(f),
            Self::Dice(d) => d.fmt(f),
        }
    }
}

/// An evaluated expression.
///
/// This is the root of the expression tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expression {
    pub(crate) roll: Box<Number>,
    pub(crate) comment: Option<Cow<'static, str>>,
}

impl Expression {
    /// Creates a new [`Expression`].
    #[must_use]
    pub fn new<N: Into<Number>>(roll: N, comment: Option<Cow<'static, str>>) -> Self {
        Self { roll: Box::new(roll.into()), comment }
    }

    fn number(&self) -> f64 {
        self.roll.number()
    }

    fn set(&self) -> Box<dyn Iterator<Item = &Number> + '_> {
        self.roll.set()
    }

    fn set_mut(&mut self) -> Box<dyn Iterator<Item = &mut Number> + '_> {
        self.roll.set_mut()
    }

    /// The inner roll of this expression.
    #[must_use]
    pub const fn roll(&self) -> &Number {
        &self.roll
    }

    /// The comment of this expression, if any.
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

/// A literal value.
#[derive(Clone, PartialEq)]
pub struct Literal {
    pub(crate) values: Vec<f64>,
    pub(crate) exploded: bool,
}

impl Eq for Literal {}
impl core::hash::Hash for Literal {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.values.iter().map(|v| v.to_bits()).collect::<Vec<_>>().hash(state);
        self.exploded.hash(state);
    }
}

impl Literal {
    /// Creates a new [`Literal`].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self { values: vec![value], exploded: false }
    }

    fn number(&self) -> f64 {
        self.values[self.values.len() - 1]
    }

    /// All intermediate values of this literal. The last element of this slice
    /// is the current value.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        self.values.as_ref()
    }

    const fn explode(&mut self) {
        self.exploded = true;
    }

    /// Update the value of this literal.
    pub fn update(&mut self, value: f64) {
        self.values.push(value);
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Literal {{ {} }}", self.number())
    }
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOpKind {
    /// Negation (`-`)
    Neg,
    /// No-op (`+`)
    Pos,
}

impl UnOpKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Neg => "-",
            Self::Pos => "+",
        }
    }
}

/// A unary operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnOp {
    pub(crate) op: UnOpKind,
    pub(crate) value: Box<Number>,
}

impl UnOp {
    /// Creates a new [`UnOp`].
    #[must_use]
    pub fn new<N: Into<Number>>(op: UnOpKind, value: N) -> Self {
        Self { op, value: Box::new(value.into()) }
    }

    /// The numeric value of this operation.
    #[must_use]
    fn number(&self) -> f64 {
        match self.op {
            UnOpKind::Neg => -self.value.total(),
            UnOpKind::Pos => self.value.total(),
        }
    }
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Euclidean Division (`/`)
    Div,
    /// Euclidean floor division (`//`)
    ///
    /// Like division, but discards the fractional part.
    FloorDiv,
    /// Euclidean modulo (`%`)
    Mod,
    /// Less than (`<`)
    Lt,
    /// Greater than (`>`)
    Gt,
    /// Equal to (`==`)
    Eq,
    /// Greater than or equal to (`>=`)
    Ge,
    /// Less than or equal to (`<=`)
    Le,
    /// Not equal to (`!=`)
    Ne,
}

impl BinOpKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::FloorDiv => "//",
            Self::Mod => "%",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Eq => "==",
            Self::Ge => ">=",
            Self::Le => "<=",
            Self::Ne => "!=",
        }
    }
}

/// A binary operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinOp {
    pub(crate) left: Box<Number>,
    pub(crate) op: BinOpKind,
    pub(crate) right: Box<Number>,
}

impl BinOp {
    /// Creates a new [`BinOp`].
    ///
    /// # Errors
    ///
    /// If `op` is [`Div`][BinOpKind::Div], [`FloorDiv`][BinOpKind::FloorDiv],
    /// or [`Mod`][BinOpKind::Mod] and `right` is 0, [`Error::ZeroDivision`] is
    /// returned.
    pub fn new(left: Number, op: BinOpKind, right: Number) -> Result<Self, Error<'static>> {
        if matches!(op, BinOpKind::Div | BinOpKind::FloorDiv | BinOpKind::Mod) && right.total() == 0.0 {
            return Err(Error::ZeroDivision);
        }
        let this = Self { left: Box::new(left), op, right: Box::new(right) };
        Ok(this)
    }

    #[must_use]
    fn number(&self) -> f64 {
        match self.op {
            BinOpKind::Add => self.left.total() + self.right.total(),
            BinOpKind::Sub => self.left.total() - self.right.total(),
            BinOpKind::Mul => self.left.total() * self.right.total(),
            BinOpKind::Div => self.left.total().div_euclid(self.right.total()),
            BinOpKind::FloorDiv => (self.left.total().div_euclid(self.right.total())).floor(),
            BinOpKind::Mod => self.left.total().rem_euclid(self.right.total()),
            BinOpKind::Lt => {
                (u8::from(float_ne(self.left.total(), self.right.total()) && self.left.total() < self.right.total()))
                    .into()
            }
            BinOpKind::Gt => {
                (u8::from(float_ne(self.left.total(), self.right.total()) && self.left.total() > self.right.total()))
                    .into()
            }
            BinOpKind::Eq => (u8::from(float_eq(self.left.total(), self.right.total()))).into(),
            BinOpKind::Ge => (u8::from(self.left.total() >= self.right.total())).into(),
            BinOpKind::Le => (u8::from(self.left.total() <= self.right.total())).into(),
            BinOpKind::Ne => (u8::from(float_ne(self.left.total(), self.right.total()))).into(),
        }
    }
}

/// A selector type.
///
/// Combined with a number to create a [`SetSel`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelCat {
    /// Select the lowest `n` values.
    Lo,
    /// Select the highest `n` values.
    Hi,
    /// Select values that match `n`.
    Lit,
    /// Select values less than `n`.
    Lt,
    /// Select values greater than `n`.
    Gt,
}

impl SelCat {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Lo => "l",
            Self::Hi => "h",
            Self::Lit => "",
            Self::Lt => "<",
            Self::Gt => ">",
        }
    }
}

impl Debug for SelCat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A set selector.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetSel {
    cat: SelCat,
    num: u32,
}

impl SetSel {
    /// Creates a new [`SetSel`].
    #[must_use]
    pub const fn new(cat: SelCat, num: u32) -> Self {
        Self { cat, num }
    }

    fn lown<'a>(&self, target: &'a mut Number) -> impl Iterator<Item = &'a mut Number> + 'a {
        let mut refs = target.kept_set_mut().collect::<Vec<_>>();
        refs.sort_unstable_by(|a, b| {
            if a.total() > b.total() {
                core::cmp::Ordering::Greater
            } else if a.total() < b.total() {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Equal
            }
        });
        refs.into_iter().take(self.num as usize)
    }

    fn highn<'a>(&self, target: &'a mut Number) -> impl Iterator<Item = &'a mut Number> + 'a {
        let mut refs = target.kept_set_mut().collect::<Vec<_>>();
        refs.sort_unstable_by(|a, b| {
            if a.total() > b.total() {
                core::cmp::Ordering::Less
            } else if a.total() < b.total() {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        });
        refs.into_iter().take(self.num as usize)
    }

    fn lt<'a>(&self, target: &'a mut Number) -> impl Iterator<Item = &'a mut Number> + 'a {
        let n = f64::from(self.num);
        target.kept_set_mut().filter(move |num| num.total() < n)
    }

    fn gt<'a>(&self, target: &'a mut Number) -> impl Iterator<Item = &'a mut Number> + 'a {
        let n = f64::from(self.num);
        target.kept_set_mut().filter(move |num| num.total() > n)
    }

    fn lit<'a>(&self, target: &'a mut Number) -> impl Iterator<Item = &'a mut Number> + 'a {
        let n = f64::from(self.num);
        target.kept_set_mut().filter(move |num| float_eq(num.total(), n))
    }

    fn select(self, target: &mut Number, max_targets: Option<usize>) -> HashSet<usize> {
        if let Some(max_targets) = max_targets {
            match self.cat {
                SelCat::Lo => self.lown(target).take(max_targets).map(|n| n.hash_key).collect(),
                SelCat::Hi => self.highn(target).take(max_targets).map(|n| n.hash_key).collect(),
                SelCat::Lt => self.lt(target).take(max_targets).map(|n| n.hash_key).collect(),
                SelCat::Gt => self.gt(target).take(max_targets).map(|n| n.hash_key).collect(),
                SelCat::Lit => self.lit(target).take(max_targets).map(|n| n.hash_key).collect(),
            }
        } else {
            match self.cat {
                SelCat::Lo => self.lown(target).map(|n| n.hash_key).collect(),
                SelCat::Hi => self.highn(target).map(|n| n.hash_key).collect(),
                SelCat::Lt => self.lt(target).map(|n| n.hash_key).collect(),
                SelCat::Gt => self.gt(target).map(|n| n.hash_key).collect(),
                SelCat::Lit => self.lit(target).map(|n| n.hash_key).collect(),
            }
        }
    }
}

impl Debug for SetSel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.cat.as_str(), self.num)
    }
}

impl core::fmt::Display for SetSel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.cat.as_str(), self.num)
    }
}

/// A set operation action.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetOpKind {
    /// Keep values that match the selector.
    Keep,
    /// Drop values that match the selector.
    Drop,
    /// Reroll values that match the selector until they don't ([`Dice`] only).
    Reroll,
    /// Reroll values that match the selector once ([`Dice`] only).
    RerollOnce,
    /// Reroll values that match the selector once, and add the result to the
    /// total ([`Dice`] only).
    ExplodeOnce,
    /// Reroll values that match the selector until they don't, accumulating the
    /// total ([`Dice`] only).
    Explode,
    /// Clamp the values to the provided minimum ([`Dice`] only).
    Min,
    /// Clamp the values to the provided maximum ([`Dice`] only).
    Max,
}

impl SetOpKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Keep => "k",
            Self::Drop => "d",
            Self::Reroll => "rr",
            Self::RerollOnce => "ro",
            Self::ExplodeOnce => "ra",
            Self::Explode => "e",
            Self::Min => "mi",
            Self::Max => "ma",
        }
    }
}
impl Debug for SetOpKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A set operator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetOp {
    op: SetOpKind,
    sels: Vec<SetSel>,
}

impl SetOp {
    /// Creates a new [`SetOp`].
    #[must_use]
    pub const fn new(op: SetOpKind, sels: Vec<SetSel>) -> Self {
        Self { op, sels }
    }

    fn select(&self, target: &mut Number, max_targets: Option<usize>) -> HashSet<usize> {
        let mut out = HashSet::new();
        for sel in &self.sels {
            let batch = max_targets.map(|m| m - out.len());
            if batch == Some(0) {
                break;
            }
            out.extend(sel.select(target, batch));
        }
        out
    }

    pub(crate) fn operate<R: rand::RngExt, C: RollCounter>(
        &self,
        target: &mut Number,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        if let NumberKind::Dice(_) = target.kind() {
            match self.op {
                SetOpKind::Keep => self.keep(target),
                SetOpKind::Drop => self.drop(target),
                SetOpKind::Reroll => self.reroll(target, ctx, rng)?,
                SetOpKind::RerollOnce => self.reroll_once(target, ctx, rng)?,
                SetOpKind::ExplodeOnce => self.explode_once(target, ctx, rng)?,
                SetOpKind::Explode => self.explode(target, ctx, rng)?,
                SetOpKind::Min => self.min(target),
                SetOpKind::Max => self.max(target),
            }
        } else {
            match self.op {
                SetOpKind::Keep => self.keep(target),
                SetOpKind::Drop => self.drop(target),
                _ => unreachable!("called dice operator on non-dice"),
            }
        }
        Ok(())
    }

    fn keep(&self, target: &mut Number) {
        let set = self.select(target, None);
        for v in target.kept_set_mut() {
            if !set.contains(&v.hash_key) {
                v.drop_();
            }
        }
    }

    fn drop(&self, target: &mut Number) {
        let set = self.select(target, None);
        for v in target.kept_set_mut() {
            if set.contains(&v.hash_key) {
                v.drop_();
            }
        }
    }

    fn reroll<R: rand::RngExt, C: RollCounter>(
        &self,
        target: &mut Number,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        let mut to_reroll = self.select(target, None);
        while !to_reroll.is_empty() {
            for v in target.kept_set_mut() {
                if to_reroll.contains(&v.hash_key) {
                    let NumberKind::Die(d) = &mut v.kind else {
                        unreachable!("dice value set contained non-die somehow");
                    };
                    d.reroll(ctx, rng)?;
                }
            }
            to_reroll = self.select(target, None);
        }
        Ok(())
    }

    fn reroll_once<R: rand::RngExt, C: RollCounter>(
        &self,
        target: &mut Number,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        let set = self.select(target, None);
        for v in target.kept_set_mut() {
            if set.contains(&v.hash_key) {
                let NumberKind::Die(d) = &mut v.kind else {
                    unreachable!("dice value set contained non-die somehow");
                };
                d.reroll(ctx, rng)?;
            }
        }
        Ok(())
    }

    fn explode<R: rand::RngExt, C: RollCounter>(
        &self,
        target: &mut Number,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        let mut to_explode = self.select(target, None);
        let mut exploded = HashSet::new();
        while !to_explode.is_empty() {
            let mut rolls = 0;
            for v in target.kept_set_mut() {
                if to_explode.contains(&v.hash_key) {
                    let NumberKind::Die(d) = &mut v.kind else {
                        unreachable!("dice value set contained non-die somehow");
                    };
                    d.explode();
                    rolls += 1;
                }
            }
            let NumberKind::Dice(dice) = &mut target.kind else {
                unreachable!("explode() was called on a non-dice");
            };
            for _ in 0..rolls {
                dice.roll_another(ctx, rng)?;
            }
            exploded.extend(to_explode);
            to_explode = self.select(target, None).difference(&exploded).copied().collect();
        }
        Ok(())
    }

    fn explode_once<R: rand::RngExt, C: RollCounter>(
        &self,
        target: &mut Number,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        let set = self.select(target, None);
        let mut rolls = 0;
        for v in target.kept_set_mut() {
            if set.contains(&v.hash_key) {
                let NumberKind::Die(d) = &mut v.kind else {
                    unreachable!("dice value set contained non-die somehow");
                };
                d.explode();
                rolls += 1;
            }
        }
        let NumberKind::Dice(dice) = &mut target.kind else {
            unreachable!("explode() was called on a non-dice");
        };
        for _ in 0..rolls {
            dice.roll_another(ctx, rng)?;
        }
        Ok(())
    }

    fn min(&self, target: &mut Number) {
        let selector = self.sels[self.sels.len() - 1];
        let min = f64::from(selector.num);
        for v in target.kept_set_mut() {
            if v.number() < min {
                let NumberKind::Die(d) = &mut v.kind else {
                    unreachable!("dice value set contained non-die somehow");
                };
                d.force_value(min);
            }
        }
    }

    fn max(&self, target: &mut Number) {
        let selector = self.sels[self.sels.len() - 1];
        let max = f64::from(selector.num);
        for v in target.kept_set_mut() {
            if v.number() > max {
                let NumberKind::Die(d) = &mut v.kind else {
                    unreachable!("dice value set contained non-die somehow");
                };
                d.force_value(max);
            }
        }
    }
}

impl core::fmt::Display for SetOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for sel in &self.sels {
            write!(f, "{}{sel}", self.op.as_str())?;
        }
        Ok(())
    }
}

impl<'a> From<&'a crate::ast::SetOp> for SetOp {
    fn from(value: &'a crate::ast::SetOp) -> Self {
        Self { op: value.op, sels: value.sels.iter().map(|s| SetSel { cat: s.cat, num: s.num }).collect() }
    }
}

/// A parenthesized value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parenthetical {
    pub(crate) value: Box<Number>,
    pub(crate) ops: Vec<SetOp>,
}

impl Parenthetical {
    /// Creates a new [`Parenthetical`].
    #[must_use]
    pub fn new<N: Into<Number>>(value: N) -> Self {
        Self { value: Box::new(value.into()), ops: vec![] }
    }

    fn number(&self) -> f64 {
        self.value.total()
    }

    fn set(&self) -> Box<dyn Iterator<Item = &Number> + '_> {
        self.value.set()
    }

    fn set_mut(&mut self) -> Box<dyn Iterator<Item = &mut Number> + '_> {
        self.value.set_mut()
    }
}

/// A set of numbers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Set {
    pub(crate) values: Vec<Number>,
    pub(crate) ops: Vec<SetOp>,
}

impl Set {
    /// Creates a new [`Set`].
    ///
    /// # Errors
    ///
    /// If any dice-only operators are used, [`Error::Value`] is returned.
    pub fn new(values: Vec<Number>, ops: Vec<SetOp>) -> Result<Self, Error<'static>> {
        for op in &ops {
            if !matches!(op.op, SetOpKind::Keep | SetOpKind::Drop) {
                return Err(Error::Value("only keep and drop are valid operators for sets"));
            }
        }
        Ok(Self { values, ops })
    }

    fn set(&self) -> core::slice::Iter<'_, Number> {
        self.values.iter()
    }

    fn set_mut(&mut self) -> core::slice::IterMut<'_, Number> {
        self.values.iter_mut()
    }
}

/// A wrapper around the number of die sides.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DieSides {
    /// Percentile die (`d%`)
    Percentile,
    /// Any other die (`d{n}`)
    Literal(u32),
}

impl Debug for DieSides {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Percentile => write!(f, "%"),
            Self::Literal(n) => write!(f, "{n}"),
        }
    }
}

impl PartialEq<u32> for DieSides {
    fn eq(&self, other: &u32) -> bool {
        match self {
            Self::Percentile => false,
            Self::Literal(n) => n == other,
        }
    }
}

impl PartialEq<DieSides> for u32 {
    fn eq(&self, other: &DieSides) -> bool {
        match other {
            DieSides::Percentile => false,
            DieSides::Literal(n) => n == self,
        }
    }
}

/// A single die.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Die {
    pub(crate) sides: DieSides,
    pub(crate) values: Vec<Number>,
}

impl Die {
    fn new<R: rand::RngExt, C: RollCounter>(sides: DieSides, ctx: &mut C, rng: &mut R) -> Result<Self, Error<'static>> {
        let mut this = Self { sides, values: vec![] };
        this.add_roll(ctx, rng)?;
        Ok(this)
    }

    fn number(&self) -> f64 {
        self.values[self.values.len() - 1].total()
    }

    fn set(&self) -> core::iter::Once<&Number> {
        core::iter::once(&self.values[self.values.len() - 1])
    }

    fn set_mut(&mut self) -> core::iter::Once<&mut Number> {
        let idx = self.values.len() - 1;
        core::iter::once(&mut self.values[idx])
    }

    fn add_roll<R: rand::RngExt, C: RollCounter>(&mut self, ctx: &mut C, rng: &mut R) -> Result<(), Error<'static>> {
        if let DieSides::Literal(sides) = self.sides
            && sides < 1
        {
            return Err(Error::Value("can't roll a 0 sided die."));
        }
        ctx.count_rolls(1)?;
        let n = match self.sides {
            DieSides::Percentile => {
                let n = rng.random_range(..10u32) * 10 + rng.random_range(..10u32);
                if n == 0 { 100 } else { n }
            }
            DieSides::Literal(sides) => rng.random_range(1..=sides),
        };
        self.values.push(Number::new(NumberKind::Literal(Literal::new(f64::from(n)))));
        Ok(())
    }

    fn reroll<R: rand::RngExt, C: RollCounter>(&mut self, ctx: &mut C, rng: &mut R) -> Result<(), Error<'static>> {
        if let Some(n) = self.values.last_mut() {
            n.drop_();
        }
        self.add_roll(ctx, rng)
    }

    fn explode(&mut self) {
        if let Some(n) = self.values.last_mut() {
            let NumberKind::Literal(l) = &mut n.kind else { unreachable!("die values must always be literals") };
            l.explode();
        }
    }

    fn force_value(&mut self, v: f64) {
        if let Some(n) = self.values.last_mut() {
            let NumberKind::Literal(l) = &mut n.kind else { unreachable!("die values must always be literals") };
            l.update(v);
        }
    }

    /// The number of sides on this die.
    #[must_use]
    pub const fn sides(&self) -> DieSides {
        self.sides
    }

    /// The values this die has rolled.
    #[must_use]
    pub fn values(&self) -> &[Number] {
        self.values.as_ref()
    }
}

/// A set of dice.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dice {
    pub(crate) values: Vec<Number>,
    pub(crate) ops: Vec<SetOp>,
    pub(crate) num: u32,
    pub(crate) sides: DieSides,
}

impl Dice {
    /// Creates a new [`Dice`].
    ///
    /// # Errors
    ///
    /// If `sides` has a value of 0, [`Error::Value`] is returned.
    pub fn new<R: rand::RngExt, C: RollCounter>(
        num: u32,
        sides: DieSides,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<Self, Error<'static>> {
        let values = (0..num).map(|_| Ok(Number::from(Die::new(sides, ctx, rng)?))).collect::<Result<_, _>>()?;
        Ok(Self { num, sides, values, ops: vec![] })
    }

    fn set(&self) -> core::slice::Iter<'_, Number> {
        self.values.iter()
    }

    fn set_mut(&mut self) -> core::slice::IterMut<'_, Number> {
        self.values.iter_mut()
    }

    fn roll_another<R: rand::RngExt, C: RollCounter>(
        &mut self,
        ctx: &mut C,
        rng: &mut R,
    ) -> Result<(), Error<'static>> {
        self.values.push(Number::new(NumberKind::Die(Die::new(self.sides, ctx, rng)?)));
        Ok(())
    }

    /// The number of dice in this set.
    #[must_use]
    pub const fn num(&self) -> u32 {
        self.num
    }

    /// The number of sides on the dice in this set.
    #[must_use]
    pub const fn sides(&self) -> DieSides {
        self.sides
    }
}

macro_rules! impl_from {
    ($($v:ident,)+$(,)?) => {
        $(
            impl From<$v> for Number {
                fn from(value: $v) -> Self {
                    Self::new(NumberKind::$v(value))
                }
            }
        )+
    };
}

impl_from! {
    Expression,
    Literal,
    UnOp,
    BinOp,
    Parenthetical,
    Set,
    Die,
    Dice,
}
