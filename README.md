# 🎲

A reasonably fast, powerful, and extensible dice engine for D&D, and any
other systems that need dice.

## Why?

- Easy to get started. `dice::roll` should cover most common
  cases.
- Extensible API using traits to customize behaviour like formatting
  expressions and limits on rolls.
- Execution limits built in.
- Tree-like expression representation for easy traversal.
- No panics (open an issue if you find one).

## Usage

```rust
use dice::roll;

let result = roll("1d20 + 5")?;
assert!((6..=25).contains(&result.total()));
if result.crit() == dice::Crit::Success {
    assert_eq!(result.total(), 25);
} else if result.crit() == dice::Crit::Failure {
    assert_eq!(result.total(), 6);
}
assert_eq!(result.ast().to_string(), "1d20 + 5");
```

## Syntax

The grammar supported by the syntax is as follows, in roughly binding order:

### Atoms

The smallest syntactical units of an expression are either literal numbers
(integer or floating point), or dice expressions in the form of `XdY`. `X`
is optional and defaults to 1 if not given.

A set of expressions can be wrapped in parentheses (e.g. `(1d20, 3.14,
42)`).

Expressions can be wrapped in parentheses to override operator precedence.

Note that therefore, a single expression wrapped in parentheses is *not* a
set with one element. To create a set with one element, include a trailing
comma. This mirrors rust's syntax for tuples.

### Set operations

Operations can be performed on sets and dice. These operations are expressed
as `<operator> <selector>` pairs. `<operator>` is applied on each element
that matches `<selector>`. Some operators can only be applied to dice. See
below for more information.

Parenthesized expressions are allowed to have associated set operations as
well. These operations are applied on the value set of the inner expression.
For example, `((6d6kh5)kl2)kh1` will apply `kl2` to the set of five values
returned by `6d6kh5`, and then apply `kh1` to the set of two values returned
by the previous `kl2`.

#### Operators

These operators work on both sets and dice.

| Operator | Syntax |         Description        |
|----------|--------|----------------------------|
| keep     | `k`    | Keep only matched elements |
| drop     | `p`    | Drop matched elements      |

These operators work with dice only.

|    Operator     | Syntax |                          Description                          |
|-----------------|--------|---------------------------------------------------------------|
| re-roll         | `rr`   | Re-roll matched dice until they don't match                   |
| re-roll once    | `ro`   | Re-roll matched dice once                                     |
| re-roll and add | `ra`   | Re-roll the first matched die once, keeping the original roll |
| explode         | `e`    | Roll another die for every die that matches                   |
| minimum         | `mi`   | Set the minimum value for each die                            |
| maximum         | `ma`   | Set the maximum value for each die                            |

#### Selectors

Selectors match elements from the kept values of dice or sets.

|   Selector   | Syntax |                          Description                           |
|--------------|--------|----------------------------------------------------------------|
| literal      | `N`    | Match values that are exactly `N`                              |
| highest      | `hN`   | Match the highest `N` values. Not supported for `mi` and `ma`  |
| lowest       | `lN`   | Match the lowest `N` values. Not supported for `mi` and `ma`   |
| greater than | `>N`   | Match values greater than `N`. Not supported for `mi` and `ma` |
| less than    | `<N`   | Match values less than `N`. Not supported for `mi` and `ma`    |

### Unary operators

Negation (`-`) is right associative and can be repeated (`--1` is the same
as `1`). `+` is also accepted and is a no-op.

### Binary operators

Addition (`+`), subtraction (`-`), multiplication (`*`), division (`/`),
integer division (`//`), and modulo (`%`) are supported. PEMDAS precedence
applies.

<div class="warning">

  All division is Euclidean, i.e. for negative numbers, the result is
  rounded away from zero. In Rust terms, this uses
  `div_euclid` and `rem_euclid`
  instead of the `/` and `%` operators.

</div>

Additionally, comparison operators (`<`, `<=`, `>`, `>=`, `==`, `!=`) are
supported and have the lowest precedence. Their results are coerced to
`1` for `true` and `0` for `false`.

## Example expressions

```rust
use dice::roll;

let mut result = roll("4d6kh3")?; // highest 3 of 4 6-sided dice
assert!((3..=18).contains(&result.total()));

result = roll("2d6ro<3")?; // roll 2d6s, then re-roll any 1s or 2s once

result = roll("8d6mi2")?; // roll 8d6s, clamping each roll to a minimum of 2
assert!(!(1..=3).contains(&result.total()));

result = roll("(1d4 + 1, 3, 2d6kl1)kh1")?; // the highest of 1d4+1, 3, and the lower of two d6s.
assert!((3..=6).contains(&result.total()));
```

## Custom formatting

By default, the result of each roll is formatted as a simple string, making
no distinction between kept and dropped values, crits, etc.

To change this behaviour, you can implement the `Formatter` trait,
overriding the specific `format_*` methods you want to customize, and create
a new `Roller` that uses your formatter.

```rust
use dice::{
    Formatter, Roller,
    expr::{Expression, Number},
};

#[derive(Debug, Clone, Copy)]
struct MyFormatter;

impl Formatter for MyFormatter {
    fn format_number(&self, node: &Number) -> String {
        if !node.kept() {
            return String::from("\u{274c}");
        }
        self.format_generic(node)
    }

    fn format_expression(&self, node: &Expression) -> String {
        format!(
            "The result of the roll {} was {}",
            self.format_number(node.roll()),
            node.roll().total()
        )
    }
}

let roller = Roller::builder().formatter(MyFormatter).build();
let result = roller.roll("(1, 2, 3, 4, 5)kh3")?;
assert_eq!(result.to_string(), "The result of the roll (❌, ❌, 3, 4, 5)kh3 was 12");
```

## Annotations and comments

Each expression node supports value annotations. Annotations are a way to
tag parts of an expression with additional information.

```rust
let mut result = roll("3d6 [fire] + 1d4 [piercing]")?;
println!("{result}");
// example: "3d6 (4, 3, 3) [fire] + 1d4 (4) [piercing] = 14"

result = roll("-(1d8 + 3) [healing]")?;
println!("{result}");
// example: "-(1d8 (2) + 3) [healing] = -5"

result = roll("(1 [one], 2 [two], 3 [three])")?;
assert_eq!(result.to_string(), "(1 [one], 2 [two], 3 [three]) = 6");
```

Annotations are purely visual and do not affect the evaluation of a roll.

Parsing comments is optional. Support for this can be enabled in many ways:

- Globally, using a builder, see `RollerBuilder::allow_comments`.
- For individual rolls, using `roll_with_comments`, or setting
  `allow_comments` to `true` in `roll_with_options`.
- For individual expressions, using `parse` followed by one of the
  `roll_expr` functions.

When this is enabled, the result of a roll may have a comment. A comment is
free text attached to the end of an expression. Any trailing character
sequence that cannot be otherwise parsed is parsed as a comment.

```rust
use dice::roll_with_comments;

let result = roll_with_comments("1d20 I rolled a d20")?;
assert!((1..=20).contains(&result.total()));
assert_eq!(result.comment(), Some("I rolled a d20"));
```

## Traversing the result tree

The raw results of rolls are represented as `Number`
enums. The root node is always of [kind][crate::expr::Number::kind]
`NumberKind::Expression`.

The raw parse result is of type `ast::Expression`. Manipulating
this tree in place will not affect the result of the roll. To re-evaluate
a modified tree, use `Roller::roll_expr`.

Both `ast::Node` and `expr::Number` implement the
`HasChildren` trait, which allows you to traverse
them like a normal tree. The `children` and
`children_mut` methods return an iterator
over the children that constitute a node from left to right, each of which
may have children of their own. This is well suited for operations like
locating a specific node, finding the leftmost or rightmost leaf, changing
the tree to include resistance or other modifiers, etc.

You may implement this trait on your own types to allow such operations on
them, including the utilities from the `utils` module.

### Example traversals

```rust
use dice::{ast::HasChildren, expr, roll};

let binop = roll("1 + 2 + 3 + 4")?;
let mut left = binop.expression();
while let Some(node) = left.left() {
    left = node;
}
assert_eq!(left.kind(), &expr::NumberKind::Literal(expr::Literal::new(1.0)));
```

The above pattern is also readily available in the `utils` module:

```rust
use dice::{expr, roll, utils};

let binop = roll("1 + 2 + 3 + 4")?;
let left = utils::leftmost(binop.expression());
assert_eq!(left.kind(), &expr::NumberKind::Literal(expr::Literal::new(1.0)));
```

Searching for a specific dice:

```rust
use dice::{expr, roll, utils, Formatter};

let mixed = roll("-1d8 + 4 - (3, 1d4)kh1")?;
let root = mixed.expression();
let result = utils::dfs(root, |n| matches!(n.kind(), expr::NumberKind::Dice(d) if d.num() == 1 && d.sides() == 4));
assert!(result.is_some());
assert!(dice::SimpleFormatter.format(result.unwrap()).starts_with("1d4 "));
```

## Features

This library supports some optional features to enable various RNGs. If you
wish to use a different RNG instead, consider not enabling the default
features.

- `caching`: Cache parsing results in a `quick_cache::sync::Cache`. This
  may improve performance when rolling the same expressions multiple times.
  Note that when parsing with comments, the cache is always bypassed.
- `pcg` (enabled by default): Use `rand_pcg::Pcg64Dxsm` as the default
  RNG. (implies `sys_rng`, as system entropy is used to seed the RNG)
- `sys_rng`: Use `rand::rngs::SysRng` as the default RNG.
- `xoshiro`: Use `rand_xoshiro::Xoshiro256PlusPlus` as the default RNG.
  (overrides `pcg`, implies `sys_rng` as system entropy is used to seed the
  RNG)
