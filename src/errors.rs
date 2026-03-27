use crate::utils::IteratorExt;

/// Error type for the dice parser and evaluator
#[derive(Debug, PartialEq, Eq)]
pub enum Error<'src> {
    /// A syntax error
    ///
    /// This is a vector of [`chumsky::error::Rich`] errors, which contain the
    /// expected and found tokens, as well as the span of the error.
    Syntax(Vec<chumsky::error::Rich<'src, char, chumsky::span::SimpleSpan>>),
    /// A value error returned while evaluating the AST.
    Value(&'static str),
    /// Too many dice were rolled.
    RollLimitExceeded,
    /// An attempt to divide by zero.
    ZeroDivision,
}

impl core::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Syntax(e) => {
                if e.is_empty() {
                    return write!(f, "unexpected EOF");
                }
                for e in e {
                    let expected = e.expected().join(", ");
                    if e.found().is_some_and(char::is_ascii_digit) && expected.contains("integer") {
                        write!(f, "unexpected input at {:?}: integer out of range for u32", e.span().into_range())?;
                    } else {
                        write!(
                            f,
                            "unexpected input at {:?}: expected {}, got {}",
                            e.span().into_range(),
                            expected,
                            e.found().map_or_else(|| "<EOF>".to_string(), ToString::to_string)
                        )?;
                    }
                }
                Ok(())
            }
            Self::Value(msg) => write!(f, "{msg}"),
            Self::RollLimitExceeded => {
                write!(f, "roll limit exceeded")
            }
            Self::ZeroDivision => write!(f, "attempt to divide by zero"),
        }
    }
}

impl core::error::Error for Error<'_> {}
