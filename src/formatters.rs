use crate::{
    expr::{BinOp, Dice, Die, Expression, Literal, Number, NumberKind, Parenthetical, Set, SetOp, UnOp},
    utils::IteratorExt,
};
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// A trait for formatting expressions
///
/// This trait has no required methods, so implementors can only implement
/// overrides for specific node formatting.
///
/// Formatters passed into [`Roller`][crate::Roller] should be [`Clone`], and
/// are cloned on every roll.If your type is particularly expensive to clone,
/// consider wrapping it in [`Arc`][std::sync::Arc] or [`Rc`][std::rc::Rc].
/// There are automatic implementations for both.
pub trait Formatter: core::fmt::Debug {
    /// Format an [`Expression`]
    #[inline]
    fn format_expression(&self, expr: &Expression) -> String {
        format!("{} = {}", self.format_number(&expr.roll), expr.roll.total())
    }

    /// Format a [`Literal`]
    #[inline]
    fn format_literal(&self, literal: &Literal) -> String {
        let mut history = literal.values.iter().join(" -> ");
        if literal.exploded {
            history.push('!');
        }
        history
    }

    /// Format a [`UnOp`]
    #[inline]
    fn format_unop(&self, unop: &UnOp) -> String {
        format!("{}{}", unop.op.as_str(), self.format_number(&unop.value))
    }

    /// Format a [`BinOp`]
    #[inline]
    fn format_binop(&self, binop: &BinOp) -> String {
        format!("{} {} {}", self.format_number(&binop.left), binop.op.as_str(), self.format_number(&binop.right))
    }

    /// Format a [`Parenthetical`]
    #[inline]
    fn format_parenthetical(&self, parenthetical: &Parenthetical) -> String {
        format!("({}){}", self.format_number(&parenthetical.value), format_ops(&parenthetical.ops))
    }

    /// Format a [`Set`]
    #[inline]
    fn format_set(&self, set: &Set) -> String {
        let out = set.values.iter().map(|n| self.format_number(n)).join(", ");
        if set.values.len() == 1 {
            format!("({out},){}", format_ops(&set.ops))
        } else {
            format!("({out}){}", format_ops(&set.ops))
        }
    }

    /// Format a [`Dice`]
    #[inline]
    fn format_dice(&self, dice: &Dice) -> String {
        let dice_str = dice.values.iter().map(|n| self.format_number(n)).join(", ");
        format!("{}d{:?}{} ({dice_str})", dice.num, dice.sides, format_ops(&dice.ops))
    }

    /// Format a [`Die`]
    #[inline]
    fn format_die(&self, die: &Die) -> String {
        die.values.iter().map(|n| self.format_number(n)).join(", ")
    }

    /// Format any [`Number`]. Callers should use [`format`][Formatter::format]
    /// instead.
    ///
    /// This method is called whenever a [`Number`] is encountered and should
    /// delegate to the other `format_*` methods based on the inner
    /// [`NumberKind`].
    #[inline]
    fn format_number(&self, node: &Number) -> String {
        self.format_generic(node)
    }

    /// This is a convenience method that keeps the default implementation of
    /// [`format_number`][Formatter::format_number] accessible in case
    /// downstream implementors want to reuse it.
    #[inline]
    fn format_generic(&self, node: &Number) -> String {
        let mut inner = match node.kind() {
            NumberKind::Expression(e) => self.format_expression(e),
            NumberKind::Literal(l) => self.format_literal(l),
            NumberKind::UnOp(u) => self.format_unop(u),
            NumberKind::BinOp(b) => self.format_binop(b),
            NumberKind::Parenthetical(p) => self.format_parenthetical(p),
            NumberKind::Set(s) => self.format_set(s),
            NumberKind::Dice(d) => self.format_dice(d),
            NumberKind::Die(d) => self.format_die(d),
        };
        if let Some(s) = node.annotation() {
            inner.push(' ');
            inner.push_str(s);
        }
        inner
    }

    /// Format any [`Number`]
    ///
    /// This is the method that any caller should use. The default
    /// implementation simply calls [`format_number`][Formatter::format_number],
    /// but implementors can have additional logic in here to reset any internal
    /// state.
    #[inline]
    fn format(&self, node: &Number) -> String {
        self.format_number(node)
    }
}

#[inline]
fn format_ops(ops: &[SetOp]) -> String {
    ops.iter().map(ToString::to_string).join("")
}

/// A simple formatter that uses only the provided methods of [`Formatter`]
#[derive(Debug, Clone, Copy)]
pub struct SimpleFormatter;

impl Formatter for SimpleFormatter {}

#[derive(Debug, Default)]
struct MardownFormatterCtx {
    in_dropped: AtomicBool,
    lock: std::sync::Mutex<()>,
}

impl MardownFormatterCtx {
    #[inline]
    const fn new() -> Self {
        Self { in_dropped: AtomicBool::new(false), lock: std::sync::Mutex::new(()) }
    }

    #[inline]
    fn reset(&self) -> std::sync::MutexGuard<'_, ()> {
        let guard = self.lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        self.in_dropped.store(false, Relaxed);
        guard
    }
}

/// A formatter that uses markdown to format the output
///
/// This isn't [`Clone`], and therefore should be wrapped in
/// [`Arc`][std::sync::Arc] or [`Rc`][std::rc::Rc] when using in a
/// [`Roller`][crate::Roller].
#[derive(Debug, Default)]
pub struct MarkdownFormatter {
    ctx: MardownFormatterCtx,
}

impl MarkdownFormatter {
    /// Creates a new [`MarkdownFormatter`]
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self { ctx: MardownFormatterCtx::new() }
    }
}

impl Formatter for MarkdownFormatter {
    #[inline]
    fn format(&self, node: &Number) -> String {
        // ensure that no one else is mutating
        let _guard = self.ctx.reset();
        self.format_number(node)
    }

    #[inline]
    fn format_number(&self, node: &Number) -> String {
        if !node.kept() && !self.ctx.in_dropped.load(Relaxed) {
            self.ctx.in_dropped.store(true, Relaxed);
            let inner = self.format_generic(node);
            self.ctx.in_dropped.store(false, Relaxed);
            format!("~~{inner}~~")
        } else {
            self.format_generic(node)
        }
    }

    #[inline]
    fn format_expression(&self, expr: &Expression) -> String {
        format!("{} = `{}`", self.format_number(&expr.roll), expr.roll.total())
    }

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[inline]
    fn format_die(&self, die: &Die) -> String {
        die.values
            .iter()
            .map(|v| {
                let mut inner = self.format_number(v);
                if v.number() as u32 == 1 || die.sides() == v.number() as u32 {
                    inner = format!("**{inner}**");
                }
                inner
            })
            .join(", ")
    }
}

macro_rules! impl_wrapper {
    ($v:path, $m:ident) => {
        impl<T: Formatter> Formatter for $v {
            #[inline]
            fn format_expression(&self, expr: &Expression) -> String {
                self.$m().format_expression(expr)
            }

            #[inline]
            fn format_literal(&self, literal: &Literal) -> String {
                self.$m().format_literal(literal)
            }

            #[inline]
            fn format_unop(&self, unop: &UnOp) -> String {
                self.$m().format_unop(unop)
            }

            #[inline]
            fn format_binop(&self, binop: &BinOp) -> String {
                self.$m().format_binop(binop)
            }

            #[inline]
            fn format_parenthetical(&self, parenthetical: &Parenthetical) -> String {
                self.$m().format_parenthetical(parenthetical)
            }

            #[inline]
            fn format_set(&self, set: &Set) -> String {
                self.$m().format_set(set)
            }

            #[inline]
            fn format_dice(&self, dice: &Dice) -> String {
                self.$m().format_dice(dice)
            }

            #[inline]
            fn format_die(&self, die: &Die) -> String {
                self.$m().format_die(die)
            }

            #[inline]
            fn format_number(&self, node: &Number) -> String {
                self.$m().format_number(node)
            }

            #[inline]
            fn format(&self, node: &Number) -> String {
                self.$m().format(node)
            }
        }
    };
}

impl_wrapper! { std::sync::Arc<T>, as_ref }
impl_wrapper! { Box<T>, as_ref }
impl_wrapper! { std::rc::Rc<T>, as_ref }

// support std::sync::OnceLock<T> etc.
impl<T: Formatter> Formatter for &T {
    #[inline]
    fn format_expression(&self, expr: &Expression) -> String {
        (*self).format_expression(expr)
    }

    #[inline]
    fn format_literal(&self, literal: &Literal) -> String {
        (*self).format_literal(literal)
    }

    #[inline]
    fn format_unop(&self, unop: &UnOp) -> String {
        (*self).format_unop(unop)
    }

    #[inline]
    fn format_binop(&self, binop: &BinOp) -> String {
        (*self).format_binop(binop)
    }

    #[inline]
    fn format_parenthetical(&self, parenthetical: &Parenthetical) -> String {
        (*self).format_parenthetical(parenthetical)
    }

    #[inline]
    fn format_set(&self, set: &Set) -> String {
        (*self).format_set(set)
    }

    #[inline]
    fn format_dice(&self, dice: &Dice) -> String {
        (*self).format_dice(dice)
    }

    #[inline]
    fn format_die(&self, die: &Die) -> String {
        (*self).format_die(die)
    }

    #[inline]
    fn format_number(&self, node: &Number) -> String {
        (*self).format_number(node)
    }

    #[inline]
    fn format(&self, node: &Number) -> String {
        (*self).format(node)
    }
}
