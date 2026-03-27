//! Utility functions mainly for working with tree-like structures.

use crate::{
    Advantage,
    ast::{self, HasChildren},
};

/// Returns the [`ast::Node`] with the given [`Advantage`] applied, if
/// applicable.
///
/// # Example
///
/// ```rust
/// # use dice::{Advantage, utils::ast_with_adv};
/// # fn main() -> Result<(), dice::Error<'static>> {
/// let tree = dice::parse("1d20 + 5", false)?;
/// assert_eq!(format!("{tree}"), "1d20 + 5");
/// let with_adv = ast_with_adv(tree.into(), Advantage::Advantage);
/// assert_eq!(format!("{with_adv}"), "2d20kh1 + 5");
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn ast_with_adv(mut root: ast::Node, adv: Advantage) -> ast::Node {
    let mut dice = &root;
    let mut level = 0;
    // avoid borrowing mutably until the end
    // keep track of how deep the ast is first...
    while let Some(d) = dice.left() {
        if d.left().is_none() {
            break;
        }
        level += 1;
        dice = d;
    }
    let ast::Node::Dice(d) = dice else { return root };
    if d.sides() == crate::expr::DieSides::Literal(20) && d.num() == 1 {
        let hilo = match adv {
            Advantage::Advantage => crate::expr::SelCat::Hi,
            Advantage::Disadvantage => crate::expr::SelCat::Lo,
            Advantage::None => return root,
        };
        let mut d = d.clone();
        let op = ast::SetOp::new(crate::expr::SetOpKind::Keep, ast::SetSel::new(hilo, 1));
        d.value = Box::new(ast::DiceExpr::new(2, crate::expr::DieSides::Literal(20)).into());
        d.ops.insert(0, op);
        let mut target = &mut root;
        // ... and now, descend to the correct level and get a mutable reference
        // to the dice node we found earlier
        for _ in 0..level {
            target = target.left_mut().unwrap_or_else(|| unreachable!());
        }
        *target = ast::Node::Dice(d);
    }
    root
}

/// Returns the first node in the tree that matches the predicate `f`.
///
/// The tree is searched depth-first, left-to-right.
///
/// If no node matches, [`None`] is returned.
pub fn dfs<T: HasChildren<Item = T>, F: Fn(&T) -> bool + Copy>(node: &T, f: F) -> Option<&T> {
    if f(node) {
        return Some(node);
    }
    for child in node.children() {
        if let Some(res) = dfs(child, f) {
            return Some(res);
        }
    }
    None
}

/// Returns the rightmost node in the tree.
pub fn rightmost<T: HasChildren<Item = T>>(tree: &T) -> &T {
    let mut right = tree;
    while let Some(r) = right.right() {
        right = r;
    }
    right
}

/// Returns the leftmost node in the tree.
pub fn leftmost<T: HasChildren<Item = T>>(tree: &T) -> &T {
    let mut left = tree;
    while let Some(l) = left.left() {
        left = l;
    }
    left
}

/// Returns a new tree with `f` applied to each node recursively.
pub fn map_tree<T: Clone + HasChildren<Item = T>, F: Fn(T) -> T + Copy>(tree: &T, f: F) -> T {
    let mut new = tree.clone();
    for (i, child) in tree.children().enumerate() {
        new.set_child(i, map_tree(child, f));
    }
    f(new)
}

#[inline]
pub(crate) fn float_eq(a: f64, b: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        return false;
    }
    if a.is_infinite() || b.is_infinite() {
        // we only arrive here because of overflow, so we assume inequality
        return false;
    }
    if a == b {
        return true;
    }
    let abs_a = a.abs();
    let abs_b = b.abs();
    let diff = (a - b).abs();
    if a == 0.0 || b == 0.0 || (abs_a + abs_b) < f64::MIN_POSITIVE {
        diff < (f64::EPSILON * f64::MIN_POSITIVE)
    } else {
        diff / (abs_a + abs_b).min(f64::MAX) < f64::EPSILON
    }
}

#[inline]
pub(crate) fn float_ne(a: f64, b: f64) -> bool {
    !float_eq(a, b)
}

pub(crate) trait IteratorExt: Iterator {
    fn join(&mut self, sep: &str) -> String
    where
        Self::Item: core::fmt::Display,
    {
        use core::fmt::Write;
        macro_rules! write {
            ($f:expr, $($arg:tt)*) => {
                ::core::write!($f, $($arg)*).unwrap()
            };
        }
        self.next().map_or_else(String::new, |first| {
            let (cap, _) = self.size_hint();
            let mut ret = String::with_capacity(sep.len() * cap);
            write!(&mut ret, "{first}");
            self.for_each(|v| {
                write!(&mut ret, "{sep}{v}");
            });
            ret
        })
    }
}

impl<I: Iterator> IteratorExt for I {}
