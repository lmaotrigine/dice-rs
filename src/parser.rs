use crate::ast;
use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    prelude::{any, choice, end, just, none_of, one_of, recursive},
    span::SimpleSpan,
    text,
};

// trait alias
pub trait ParserLike<'src, T>: Parser<'src, &'src str, T, extra::Err<Rich<'src, char, SimpleSpan>>> + Clone {}

impl<'src, T, U> ParserLike<'src, U> for T where
    T: Parser<'src, &'src str, U, extra::Err<Rich<'src, char, SimpleSpan>>> + Clone
{
}

#[inline]
fn integer<'src>() -> impl ParserLike<'src, u32> {
    text::int(10)
        .from_str()
        .try_map(|v, span| v.map_err(|_| Rich::custom(span, "integer value overflows u32")))
        .labelled("integer")
}

#[inline]
fn literal<'src>() -> impl ParserLike<'src, ast::Literal> {
    let integer = text::int(10).from_str().try_map(|v, span| {
        v.map_or_else(|_| Err(Rich::custom(span, "literal value overflows f64")), |v| Ok(ast::Literal::new(v)))
    });
    let decimal =
        text::int(10).or_not().then(just('.').then(text::digits(10))).to_slice().from_str().try_map(|v, span| {
            v.map_or_else(|_| Err(Rich::custom(span, "literal value overflows f64")), |v| Ok(ast::Literal::new(v)))
        });
    decimal.or(integer)
}

#[inline]
fn selector<'src>() -> impl ParserLike<'src, ast::SetSel> {
    let cat = one_of("lh<>").or_not().map(|c| match c {
        Some('l') => ast::SelCat::Lo,
        Some('h') => ast::SelCat::Hi,
        Some('<') => ast::SelCat::Lt,
        Some('>') => ast::SelCat::Gt,
        None => ast::SelCat::Lit,
        _ => unreachable!(),
    });
    cat.then(integer()).map(|(cat, num)| ast::SetSel { cat, num }).labelled("selector")
}

#[inline]
fn set_op<'src>() -> impl ParserLike<'src, ast::SetOp> {
    let op = one_of("kp").map(|c| match c {
        'k' => ast::SetOpKind::Keep,
        'p' => ast::SetOpKind::Drop,
        _ => unreachable!(),
    });
    op.then(selector()).map(|(op, sel)| ast::SetOp { op, sels: vec![sel] }).labelled("selector")
}

#[inline]
fn dice_expr<'src>() -> impl ParserLike<'src, ast::DiceExpr> {
    let sides =
        just('d').ignore_then(integer().map(ast::DieSides::Literal).or(just('%').to(ast::DieSides::Percentile)));
    integer()
        .then(sides.clone())
        .map(|(num, sides)| ast::DiceExpr { num, sides })
        .or(sides.map(|sides| ast::DiceExpr { num: 1, sides }))
}

#[inline]
fn dice_op<'src>() -> impl ParserLike<'src, ast::SetOp> {
    let opsel = choice((
        choice((
            just("rr").to(ast::SetOpKind::Reroll),
            just("ro").to(ast::SetOpKind::RerollOnce),
            just("ra").to(ast::SetOpKind::ExplodeOnce),
            just("e").to(ast::SetOpKind::Explode),
            just("k").to(ast::SetOpKind::Keep),
            just("p").to(ast::SetOpKind::Drop),
        ))
        .then(selector()),
        choice((just("mi").to(ast::SetOpKind::Min), just("ma").to(ast::SetOpKind::Max)))
            .then(integer().map(|num| ast::SetSel { cat: ast::SelCat::Lit, num })),
    ));
    opsel.map(|(op, sel)| ast::SetOp { op, sels: vec![sel] })
}

#[inline]
fn dice<'src>() -> impl ParserLike<'src, ast::Dice> {
    dice_expr().then(dice_op().repeated().collect::<Vec<_>>()).map(|(expr, ops)| ast::Dice::new(expr, &ops))
}

#[inline]
fn parser<'src>() -> impl ParserLike<'src, ast::Node> {
    recursive(|expr| {
        let literal = literal().labelled("literal");
        let items = expr.clone().separated_by(just(',').padded()).allow_trailing().collect::<Vec<_>>();
        let set = items
            .clone()
            .map(|s| ast::Node::SetExpr(ast::SetExpr { values: s }))
            .delimited_by(just('('), just(')'))
            .then(set_op().repeated().collect::<Vec<_>>())
            .map(|(set, ops)| ast::Set::new(set, &ops).into());
        let num_expr = expr
            .clone()
            .padded()
            .delimited_by(just('('), just(')'))
            .then(set_op().repeated().collect::<Vec<_>>())
            .map(|(p, ops)| {
                let node = ast::Parenthetical::new(p);
                if ops.is_empty() { node.into() } else { ast::Set::new(node, &ops).into() }
            })
            .or(dice().padded().map(ast::Node::Dice))
            .or(set.padded())
            .or(literal.padded().map(ast::Node::Literal));
        let atom = num_expr
            .then(
                just('[')
                    .then(none_of(']').repeated().collect::<String>())
                    .then(just(']'))
                    .to_slice()
                    .padded()
                    .repeated()
                    .collect::<Vec<_>>()
                    .padded(),
            )
            .map(|(node, ann)| if ann.is_empty() { node } else { ast::Annotated::new(node, ann).into() });
        let unary = just('+')
            .to(ast::UnOpKind::Pos)
            .or(just('-').to(ast::UnOpKind::Neg))
            .padded()
            .repeated()
            .foldr(atom, |op, node| ast::UnOp::new(op, node).into());
        let op = choice((
            just("//").to(ast::BinOpKind::FloorDiv),
            just('*').to(ast::BinOpKind::Mul),
            just('/').to(ast::BinOpKind::Div),
            just('%').to(ast::BinOpKind::Mod),
        ))
        .padded();
        let mul =
            unary.clone().foldl(op.then(unary).repeated(), |left, (op, right)| ast::BinOp::new(left, op, right).into());
        let op = just('+').to(ast::BinOpKind::Add).or(just('-').to(ast::BinOpKind::Sub)).padded();
        let add =
            mul.clone().foldl(op.then(mul).repeated(), |left, (op, right)| ast::BinOp::new(left, op, right).into());
        let op = choice((
            just("==").to(ast::BinOpKind::Eq),
            just("!=").to(ast::BinOpKind::Ne),
            just(">=").to(ast::BinOpKind::Ge),
            just("<=").to(ast::BinOpKind::Le),
            just('>').to(ast::BinOpKind::Gt),
            just('<').to(ast::BinOpKind::Lt),
        ))
        .padded();
        let comparison =
            add.clone().foldl(op.then(add).repeated(), |left, (op, right)| ast::BinOp::new(left, op, right).into());
        comparison.labelled("expression")
    })
}

pub fn commented_expr<'src>() -> impl ParserLike<'src, ast::Expression> {
    parser().then(any().repeated().collect::<String>()).map(|(node, comment)| {
        let comment = comment.trim().to_string();
        if comment.is_empty() {
            ast::Expression::new(node, None::<&'static str>)
        } else {
            ast::Expression::new(node, Some(comment))
        }
    })
}

pub fn expr<'src>() -> impl ParserLike<'src, ast::Expression> {
    parser().padded().then_ignore(end()).map(|n| ast::Expression::new(n, None::<&'static str>))
}
