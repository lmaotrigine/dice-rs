//! Types for the abstract syntax tree (AST) returned by the parser.

pub(crate) use crate::expr::{BinOpKind, DieSides, SelCat, SetOpKind, UnOpKind};
use crate::utils::IteratorExt;
use core::fmt::Display;
use std::borrow::Cow;

/// A trait for types that are nodes in a tree.
///
/// This is implemented for the AST and the expression tree.
///
/// It is not sealed, so you may implement it for any other type in order to use
/// the utility functions in [`crate::utils`].
pub trait HasChildren {
    /// The type of each child node.
    type Item;
    /// The return type of [`children`][HasChildren::children].
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;
    /// The return type of [`children_mut`][HasChildren::children_mut].
    type IterMut<'a>: Iterator<Item = &'a mut Self::Item>
    where
        Self: 'a;

    /// Returns an iterator over references to the child nodes.
    fn children(&self) -> Self::Iter<'_>;
    /// Returns an iterator over mutable references to the child nodes.
    fn children_mut(&mut self) -> Self::IterMut<'_>;
    /// Sets the child node at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn set_child<T: Into<Self::Item>>(&mut self, idx: usize, child: T);
    /// Returns a reference to the left child node, if any.
    fn left(&self) -> Option<&Self::Item>;
    /// Returns a reference to the right child node, if any.
    fn right(&self) -> Option<&Self::Item>;
    /// Sets the left child node.
    fn set_left<T: Into<Self::Item>>(&mut self, child: T);
    /// Sets the right child node.
    fn set_right<T: Into<Self::Item>>(&mut self, child: T);
}

/// A node in the abstract syntax tree.
#[derive(Clone, PartialEq, Eq)]
pub enum Node {
    /// A roll expression.
    ///
    /// This is always the root of any parsed expression.
    Expression(Expression),
    /// A literal value.
    Literal(Literal),
    /// A parenthetical value.
    Parenthetical(Parenthetical),
    /// An annotated value.
    Annotated(Annotated),
    /// A unary operation.
    UnOp(UnOp),
    /// A binary operation.
    BinOp(BinOp),
    /// A set expression
    SetExpr(SetExpr),
    /// A dice expression
    DiceExpr(DiceExpr),
    /// A set expression along with operators.
    Set(Set),
    /// A dice expression along with operators.
    Dice(Dice),
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(n) => core::fmt::Debug::fmt(n, f),
            Self::Parenthetical(n) => core::fmt::Debug::fmt(n, f),
            Self::Expression(n) => core::fmt::Debug::fmt(n, f),
            Self::Annotated(n) => core::fmt::Debug::fmt(n, f),
            Self::UnOp(n) => core::fmt::Debug::fmt(n, f),
            Self::BinOp(n) => core::fmt::Debug::fmt(n, f),
            Self::SetExpr(n) => core::fmt::Debug::fmt(n, f),
            Self::DiceExpr(n) => core::fmt::Debug::fmt(n, f),
            Self::Set(n) => core::fmt::Debug::fmt(n, f),
            Self::Dice(n) => core::fmt::Debug::fmt(n, f),
        }
    }
}

impl Node {
    pub(crate) fn left_mut(&mut self) -> Option<&mut Self> {
        self.children_mut().next()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(n) => write!(f, "{n}"),
            Self::Parenthetical(n) => write!(f, "{n}"),
            Self::Expression(n) => write!(f, "{n}"),
            Self::Annotated(n) => write!(f, "{n}"),
            Self::UnOp(n) => write!(f, "{n}"),
            Self::BinOp(n) => write!(f, "{n}"),
            Self::SetExpr(n) => write!(f, "{n}"),
            Self::DiceExpr(n) => write!(f, "{n}"),
            Self::Set(n) => write!(f, "{n}"),
            Self::Dice(n) => write!(f, "{n}"),
        }
    }
}

impl HasChildren for Node {
    type Item = Self;
    type Iter<'a> = Box<dyn Iterator<Item = &'a Self> + 'a>;
    type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Self> + 'a>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        match self {
            Self::Literal(n) => Box::new(n.children()),
            Self::Parenthetical(n) => Box::new(n.children()),
            Self::Expression(n) => Box::new(n.children()),
            Self::Annotated(n) => Box::new(n.children()),
            Self::UnOp(n) => Box::new(n.children()),
            Self::BinOp(n) => Box::new(n.children()),
            Self::SetExpr(n) => Box::new(n.children()),
            Self::DiceExpr(n) => Box::new(n.children()),
            Self::Set(n) => Box::new(n.children()),
            Self::Dice(n) => Box::new(n.children()),
        }
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        match self {
            Self::Literal(n) => Box::new(n.children_mut()),
            Self::Parenthetical(n) => Box::new(n.children_mut()),
            Self::Expression(n) => Box::new(n.children_mut()),
            Self::Annotated(n) => Box::new(n.children_mut()),
            Self::UnOp(n) => Box::new(n.children_mut()),
            Self::BinOp(n) => Box::new(n.children_mut()),
            Self::SetExpr(n) => Box::new(n.children_mut()),
            Self::DiceExpr(n) => Box::new(n.children_mut()),
            Self::Set(n) => Box::new(n.children_mut()),
            Self::Dice(n) => Box::new(n.children_mut()),
        }
    }

    #[inline]
    fn left(&self) -> Option<&Self::Item> {
        match self {
            Self::Literal(n) => n.left(),
            Self::Parenthetical(n) => n.left(),
            Self::Expression(n) => n.left(),
            Self::Annotated(n) => n.left(),
            Self::UnOp(n) => n.left(),
            Self::BinOp(n) => n.left(),
            Self::SetExpr(n) => n.left(),
            Self::DiceExpr(n) => n.left(),
            Self::Set(n) => n.left(),
            Self::Dice(n) => n.left(),
        }
    }

    #[inline]
    fn right(&self) -> Option<&Self::Item> {
        match self {
            Self::Literal(n) => n.right(),
            Self::Parenthetical(n) => n.right(),
            Self::Expression(n) => n.right(),
            Self::Annotated(n) => n.right(),
            Self::UnOp(n) => n.right(),
            Self::BinOp(n) => n.right(),
            Self::SetExpr(n) => n.right(),
            Self::DiceExpr(n) => n.right(),
            Self::Set(n) => n.right(),
            Self::Dice(n) => n.right(),
        }
    }

    #[inline]
    fn set_child<T: Into<Self::Item>>(&mut self, idx: usize, child: T) {
        match self {
            Self::Literal(n) => n.set_child(idx, child),
            Self::Parenthetical(n) => n.set_child(idx, child),
            Self::Expression(n) => n.set_child(idx, child),
            Self::Annotated(n) => n.set_child(idx, child),
            Self::UnOp(n) => n.set_child(idx, child),
            Self::BinOp(n) => n.set_child(idx, child),
            Self::SetExpr(n) => n.set_child(idx, child),
            Self::DiceExpr(n) => n.set_child(idx, child),
            Self::Set(n) => n.set_child(idx, child),
            Self::Dice(n) => n.set_child(idx, child),
        }
    }

    #[inline]
    fn set_left<T: Into<Self::Item>>(&mut self, child: T) {
        match self {
            Self::Literal(n) => n.set_left(child),
            Self::Parenthetical(n) => n.set_left(child),
            Self::Expression(n) => n.set_left(child),
            Self::Annotated(n) => n.set_left(child),
            Self::UnOp(n) => n.set_left(child),
            Self::BinOp(n) => n.set_left(child),
            Self::SetExpr(n) => n.set_left(child),
            Self::DiceExpr(n) => n.set_left(child),
            Self::Set(n) => n.set_left(child),
            Self::Dice(n) => n.set_left(child),
        }
    }

    #[inline]
    fn set_right<T: Into<Self::Item>>(&mut self, child: T) {
        match self {
            Self::Literal(n) => n.set_right(child),
            Self::Parenthetical(n) => n.set_right(child),
            Self::Expression(n) => n.set_right(child),
            Self::Annotated(n) => n.set_right(child),
            Self::UnOp(n) => n.set_right(child),
            Self::BinOp(n) => n.set_right(child),
            Self::SetExpr(n) => n.set_right(child),
            Self::DiceExpr(n) => n.set_right(child),
            Self::Set(n) => n.set_right(child),
            Self::Dice(n) => n.set_right(child),
        }
    }
}

/// A literal value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Literal {
    value: f64,
}

impl Eq for Literal {}

impl Literal {
    /// Creates a new [`Literal`].
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self { value }
    }

    /// The value of the literal.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl HasChildren for Literal {
    type Item = Node;
    type Iter<'a> = core::iter::Empty<&'a Node>;
    type IterMut<'a> = core::iter::Empty<&'a mut Node>;

    fn children(&self) -> Self::Iter<'_> {
        core::iter::empty()
    }

    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::empty()
    }

    fn left(&self) -> Option<&Node> {
        None
    }

    fn right(&self) -> Option<&Node> {
        None
    }

    fn set_child<T: Into<Node>>(&mut self, _idx: usize, _child: T) {}

    fn set_left<T: Into<Node>>(&mut self, _child: T) {}

    fn set_right<T: Into<Node>>(&mut self, _child: T) {}
}

impl From<Literal> for Node {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

/// A parenthesized value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parenthetical {
    pub(crate) value: Box<Node>,
}

impl Parenthetical {
    /// Creates a new [`Parenthetical`].
    #[must_use]
    pub fn new<N: Into<Node>>(value: N) -> Self {
        Self { value: Box::new(value.into()) }
    }

    /// The inner value of the parenthetical.
    #[must_use]
    pub const fn value(&self) -> &Node {
        &self.value
    }
}

impl Display for Parenthetical {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({})", self.value)
    }
}

impl HasChildren for Parenthetical {
    type Item = Node;
    type Iter<'a> = core::iter::Once<&'a Node>;
    type IterMut<'a> = core::iter::Once<&'a mut Node>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        core::iter::once(&self.value)
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::once(&mut *self.value)
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        assert_eq!(idx, 0, "index out of bounds");
        *self.value = child.into();
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }
}

impl From<Parenthetical> for Node {
    fn from(parenthetical: Parenthetical) -> Self {
        Self::Parenthetical(parenthetical)
    }
}

/// A parsed roll expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub(crate) roll: Box<Node>,
    pub(crate) comment: Option<Cow<'static, str>>,
}

impl Expression {
    /// Creates a new [`Expression`].
    pub fn new<S: Into<Cow<'static, str>>, N: Into<Node>>(roll: N, comment: Option<S>) -> Self {
        Self { roll: Box::new(roll.into()), comment: comment.map(Into::into) }
    }

    /// The roll expression.
    #[must_use]
    #[inline]
    pub const fn roll(&self) -> &Node {
        &self.roll
    }

    /// The comment.
    #[must_use]
    #[inline]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(comment) = &self.comment {
            write!(f, "{} {}", self.roll, comment)
        } else {
            write!(f, "{}", self.roll)
        }
    }
}

impl HasChildren for Expression {
    type Item = Node;
    type Iter<'a> = core::iter::Once<&'a Node>;
    type IterMut<'a> = core::iter::Once<&'a mut Node>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        core::iter::once(&self.roll)
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::once(&mut *self.roll)
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        Some(&self.roll)
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        Some(&self.roll)
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        assert_eq!(idx, 0, "index out of bounds");
        *self.roll = child.into();
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        *self.roll = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        *self.roll = child.into();
    }
}

impl From<Expression> for Node {
    fn from(expression: Expression) -> Self {
        Self::Expression(expression)
    }
}

/// An annotated value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotated {
    pub(crate) value: Box<Node>,
    pub(crate) annotations: Vec<String>,
}

impl Annotated {
    /// Creates a new [`Annotated`].
    pub fn new<N: Into<Node>, S: AsRef<str>>(value: N, annotations: Vec<S>) -> Self {
        Self {
            value: Box::new(value.into()),
            annotations: annotations.into_iter().map(|s| s.as_ref().trim().to_owned()).collect(),
        }
    }
}

impl Display for Annotated {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.value, self.annotations.join(""))
    }
}

impl HasChildren for Annotated {
    type Item = Node;
    type Iter<'a> = core::iter::Once<&'a Node>;
    type IterMut<'a> = core::iter::Once<&'a mut Node>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        core::iter::once(&self.value)
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::once(&mut *self.value)
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        assert_eq!(idx, 0, "index out of bounds");
        *self.value = child.into();
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }
}

impl From<Annotated> for Node {
    fn from(annotated: Annotated) -> Self {
        Self::Annotated(annotated)
    }
}

/// A unary operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnOp {
    pub(crate) op: UnOpKind,
    pub(crate) value: Box<Node>,
}

impl UnOp {
    /// Creates a new [`UnOp`].
    pub fn new<N: Into<Node>>(op: UnOpKind, value: N) -> Self {
        Self { op, value: Box::new(value.into()) }
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.op.as_str(), self.value)
    }
}

impl HasChildren for UnOp {
    type Item = Node;
    type Iter<'a> = core::iter::Once<&'a Node>;
    type IterMut<'a> = core::iter::Once<&'a mut Node>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        core::iter::once(&self.value)
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::once(&mut *self.value)
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        Some(&self.value)
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        assert_eq!(idx, 0, "index out of bounds");
        *self.value = child.into();
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        *self.value = child.into();
    }
}

impl From<UnOp> for Node {
    fn from(un_op: UnOp) -> Self {
        Self::UnOp(un_op)
    }
}

/// A binary operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOp {
    pub(crate) op: BinOpKind,
    pub(crate) left: Box<Node>,
    pub(crate) right: Box<Node>,
}

impl BinOp {
    /// Creates a new [`BinOp`].
    pub fn new<L: Into<Node>, R: Into<Node>>(left: L, op: BinOpKind, right: R) -> Self {
        Self { op, left: Box::new(left.into()), right: Box::new(right.into()) }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {} {}", self.left, self.op.as_str(), self.right)
    }
}

impl HasChildren for BinOp {
    type Item = Node;
    type Iter<'a> = core::array::IntoIter<&'a Node, 2>;
    type IterMut<'a> = core::array::IntoIter<&'a mut Node, 2>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        [self.left.as_ref(), self.right.as_ref()].into_iter()
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        [self.left.as_mut(), self.right.as_mut()].into_iter()
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        Some(&self.left)
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        Some(&self.right)
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        assert!(idx < 2, "index out of bounds");
        match idx {
            0 => *self.left = child.into(),
            1 => *self.right = child.into(),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        *self.left = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        *self.right = child.into();
    }
}

impl From<BinOp> for Node {
    fn from(bin_op: BinOp) -> Self {
        Self::BinOp(bin_op)
    }
}

/// A set selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetSel {
    pub(crate) cat: SelCat,
    pub(crate) num: u32,
}

impl SetSel {
    /// Creates a new [`SetSel`].
    #[must_use]
    pub const fn new(cat: SelCat, num: u32) -> Self {
        Self { cat, num }
    }
}

impl Display for SetSel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}", self.cat.as_str(), self.num)
    }
}

/// A set operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetOp {
    pub(crate) op: SetOpKind,
    pub(crate) sels: Vec<SetSel>,
}

impl SetOp {
    /// Creates a new [`SetOp`].
    #[must_use]
    pub fn new(op: SetOpKind, sel: SetSel) -> Self {
        Self { op, sels: vec![sel] }
    }

    /// Append selectors to this [`SetOp`].
    pub fn add_sels<I: IntoIterator<Item = SetSel>>(&mut self, selectors: I) {
        self.sels.extend(selectors);
    }
}

impl Display for SetOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        macro_rules! write {
            ($f:expr, $($arg:tt)*) => {
                ::core::write!($f, $($arg)*).unwrap()
            };
        }
        core::write!(
            f,
            "{}",
            self.sels.iter().fold(String::new(), |mut acc, sel| {
                write!(&mut acc, "{}{sel}", self.op.as_str());
                acc
            })
        )
    }
}

/// A set expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetExpr {
    pub(crate) values: Vec<Node>,
}

impl SetExpr {
    /// Creates a new [`SetExpr`].
    #[must_use]
    pub const fn new(values: Vec<Node>) -> Self {
        Self { values }
    }
}

impl Display for SetExpr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.values.len() == 1 {
            write!(f, "({},)", self.values[0])
        } else {
            write!(f, "({})", self.values.iter().join(", "))
        }
    }
}

impl HasChildren for SetExpr {
    type Item = Node;
    type Iter<'a> = core::slice::Iter<'a, Node>;
    type IterMut<'a> = core::slice::IterMut<'a, Node>;

    #[inline]
    fn children(&self) -> Self::Iter<'_> {
        self.values.iter()
    }

    #[inline]
    fn children_mut(&mut self) -> Self::IterMut<'_> {
        self.values.iter_mut()
    }

    #[inline]
    fn left(&self) -> Option<&Node> {
        self.values.first()
    }

    #[inline]
    fn right(&self) -> Option<&Node> {
        self.values.last()
    }

    #[inline]
    fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
        self.values[idx] = child.into();
    }

    #[inline]
    fn set_left<T: Into<Node>>(&mut self, child: T) {
        self.values[0] = child.into();
    }

    #[inline]
    fn set_right<T: Into<Node>>(&mut self, child: T) {
        let idx = self.values.len() - 1;
        self.values[idx] = child.into();
    }
}

impl From<SetExpr> for Node {
    fn from(set_expr: SetExpr) -> Self {
        Self::SetExpr(set_expr)
    }
}

/// A dice expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceExpr {
    pub(crate) num: u32,
    pub(crate) sides: DieSides,
}

impl DiceExpr {
    /// Creates a new [`DiceExpr`].
    #[must_use]
    pub const fn new(num: u32, sides: DieSides) -> Self {
        Self { num, sides }
    }

    /// The number of dice.
    #[must_use]
    pub const fn num(&self) -> u32 {
        self.num
    }

    /// The number of sides on the die.
    #[must_use]
    pub const fn sides(&self) -> DieSides {
        self.sides
    }
}

impl Display for DiceExpr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}d{:?}", self.num, self.sides)
    }
}

impl HasChildren for DiceExpr {
    type Item = Node;
    type Iter<'a> = core::iter::Empty<&'a Node>;
    type IterMut<'a> = core::iter::Empty<&'a mut Node>;

    fn children(&self) -> Self::Iter<'_> {
        core::iter::empty()
    }

    fn children_mut(&mut self) -> Self::IterMut<'_> {
        core::iter::empty()
    }

    fn left(&self) -> Option<&Node> {
        None
    }

    fn right(&self) -> Option<&Node> {
        None
    }

    fn set_child<T: Into<Node>>(&mut self, _idx: usize, _child: T) {}

    fn set_left<T: Into<Node>>(&mut self, _child: T) {}

    fn set_right<T: Into<Node>>(&mut self, _child: T) {}
}

impl From<DiceExpr> for Node {
    fn from(dice_expr: DiceExpr) -> Self {
        Self::DiceExpr(dice_expr)
    }
}

macro_rules! set_def {
    ($($ty:ident, $doc:literal, $newdoc:literal)+) => {
        $(
            #[doc = $doc]
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $ty {
                pub(crate) value: Box<Node>,
                pub(crate) ops: Vec<SetOp>,
            }

            impl $ty {
                #[doc = $newdoc]
                pub fn new<N: Into<Node>>(value: N, ops: &[SetOp]) -> Self {
                    Self {
                        value: Box::new(value.into()),
                        ops: Self::simplify(ops),
                    }
                }

                fn simplify(ops: &[SetOp]) -> Vec<SetOp> {
                    let mut new = Vec::with_capacity(ops.len());
                    for op in ops {
                        if matches!(op.op, SetOpKind::Min | SetOpKind::Max) || new.is_empty() {
                            new.push(op.clone());
                        } else {
                            let last = new.last_mut().unwrap_or_else(|| unreachable!(""));
                            if last.op == op.op {
                                last.add_sels(op.sels.iter().copied());
                            } else {
                                new.push(op.clone())
                            }
                        }
                    }
                    new
                }
            }

            impl core::fmt::Display for $ty {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}{}", self.value, self.ops.iter().join(""))
                }
            }

            impl HasChildren for $ty {
                type Item = Node;
                type Iter<'a> = core::iter::Once<&'a Node>;
                type IterMut<'a> = core::iter::Once<&'a mut Node>;

                #[inline]
                fn children(&self) -> Self::Iter<'_> {
                    core::iter::once(&self.value)
                }

                #[inline]
                fn children_mut(&mut self) -> Self::IterMut<'_> {
                    core::iter::once(&mut *self.value)
                }

                #[inline]
                fn left(&self) -> Option<&Self::Item> {
                    Some(&self.value)
                }

                #[inline]
                fn right(&self) -> Option<&Self::Item> {
                    Some(&self.value)
                }

                #[inline]
                fn set_child<T: Into<Node>>(&mut self, idx: usize, child: T) {
                    assert_eq!(idx, 0, "index out of bounds");
                    self.value = Box::new(child.into());
                }

                #[inline]
                fn set_left<T: Into<Self::Item>>(&mut self, child: T) {
                    self.value = Box::new(child.into());
                }

                #[inline]
                fn set_right<T: Into<Self::Item>>(&mut self, child: T) {
                    self.value = Box::new(child.into());
                }
            }

            impl From<$ty> for Node {
                fn from(set: $ty) -> Self {
                    Self::$ty(set)
                }
            }
        )+
    };
}

set_def! {
    Set, "A set expression along with operators.", "Creates a new [`Set`]."
    Dice, "A dice expression along with operators.", "Creates a new [`Dice`]."
}

impl Dice {
    pub(crate) fn sides(&self) -> DieSides {
        match &*self.value {
            Node::DiceExpr(d) => d.sides,
            _ => unreachable!("dice value is not a dice expression"),
        }
    }

    pub(crate) fn num(&self) -> u32 {
        match &*self.value {
            Node::DiceExpr(d) => d.num,
            _ => unreachable!("dice value is not a dice expression"),
        }
    }
}
