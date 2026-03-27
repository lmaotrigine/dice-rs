use crate::Error;

/// Counter for rolls in each expression.
///
/// A new counter is created by a [`Context`] before evaluating an AST.
pub trait RollCounter {
    /// Called whenever an operation evaluation involves one or more rolls.
    ///
    /// # Errors
    ///
    /// If no more rolls can be counted, implementations should return
    /// [`Error::RollLimitExceeded`].
    fn count_rolls(&mut self, n: usize) -> Result<(), Error<'static>>;
}

/// A context that manages [`RollCounter`]s for a [`Roller`][crate::Roller].
pub trait Context {
    /// The type of [`RollCounter`] used by this context.
    type Counter: RollCounter;

    /// Returns a new [`RollCounter`] for this context.
    ///
    /// This is called before evaluating an AST.
    fn new_counter(&self) -> Self::Counter;
    /// Called when an AST has been evaluated completely.
    fn finish(&self, counter: Self::Counter);
}

/// A [`Context`] that caps the number of rolls in a single expression.
#[derive(Debug, Clone, Copy)]
pub struct ExpressionContext {
    max_rolls: usize,
    rolls: usize,
}

impl ExpressionContext {
    /// Creates a new [`ExpressionContext`].
    #[must_use]
    pub const fn new(max_rolls: usize) -> Self {
        Self { max_rolls, rolls: 0 }
    }
}

impl RollCounter for ExpressionContext {
    fn count_rolls(&mut self, n: usize) -> Result<(), Error<'static>> {
        self.rolls += n;
        if self.rolls > self.max_rolls { Err(Error::RollLimitExceeded) } else { Ok(()) }
    }
}

impl Default for ExpressionContext {
    /// Creates a new [`ExpressionContext`] with a maximum of 1000 rolls per
    /// expression.
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Context for ExpressionContext {
    type Counter = Self;

    fn new_counter(&self) -> Self::Counter {
        *self
    }

    fn finish(&self, _counter: Self::Counter) {}
}

impl<T: RollCounter, U: core::ops::DerefMut<Target = T>> RollCounter for U {
    fn count_rolls(&mut self, n: usize) -> Result<(), Error<'static>> {
        (**self).count_rolls(n)
    }
}
