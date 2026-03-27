use dice::{
    Crit, Error,
    ast::HasChildren,
    expr::{BinOp, BinOpKind, Literal},
    roll,
};

const STANDARD_EXPRS: &[&str] = &[
    "1d20",
    "1d%",
    "1+1",
    "4d6kh3",
    "(1)",
    "(1,)",
    "((1d6))",
    "4*(3d8kh2+9[fire]+(9d2e2+3[cold])/2)",
    "(1d4, 2+2, 3d6kl1)kh1",
    "((10d6kh5)kl2)kh1",
];

fn r(e: &'static str) -> Result<isize, Error<'static>> {
    Ok(roll(e)?.total())
}

#[test]
fn test_rolls_do_not_error() -> Result<(), Error<'static>> {
    for expr in STANDARD_EXPRS {
        roll(expr)?;
    }
    Ok(())
}

#[test]
fn test_sane_totals() -> Result<(), Error<'static>> {
    for _ in 0..1000 {
        assert!((1..=20).contains(&r("1d20")?));
        assert!((1..=100).contains(&r("1d%")?));
        assert!((3..=18).contains(&r("4d6kh3")?));
        assert!((1..=6).contains(&r("((1d6))")?));
        assert!((4..=6).contains(&r("(1d4, 2+2, 3d6kl1)kh1")?));
        assert!((1..=6).contains(&r("((10d6kh5)kl2)kh1")?));
    }
    Ok(())
}

#[test]
fn test_precedence() -> Result<(), Error<'static>> {
    assert_eq!(r("1 + 3 * 6")?, r("1 + (3 * 6)")?);
    assert_eq!(r("1 + 3 * 6")?, 19);
    assert_eq!(r("(1 + 3) * 6")?, 24);
    assert_eq!(r("1 + 2 + 3")?, r("(1 + 2) + 3")?);
    assert_eq!(r("1 + 2 + 3")?, r("1 + (2 + 3)")?);
    assert_eq!(r("1 + 2 + 3")?, 6);
    assert_eq!(r("1 + 2 == 2")?, 0);
    assert_eq!(r("1 + (2 == 2)")?, 2);
    Ok(())
}

#[test]
fn test_invalid_rolls() {
    assert!(matches!(r("1001d6"), Err(Error::RollLimitExceeded)));
    assert!(matches!(r("6d0"), Err(Error::Value(_))));
    assert!(matches!(r("10d6mil1"), Err(Error::Syntax(_))));
}

#[test]
fn test_chaining() -> Result<(), Error<'static>> {
    assert!((0..=30).contains(&r("10d6k1k2k3")?));
    assert!((0..=9).contains(&r("10d6k1ph1")?));
    assert_eq!(r("(1, 2, 3)k1k2")?, 3);
    Ok(())
}

#[test]
fn test_crit() -> Result<(), Error<'static>> {
    let mut result = roll("1d20")?;
    while result.total() != 20 {
        result = roll("1d20")?;
    }
    assert_eq!(result.crit(), Crit::Success);
    while result.total() != 1 {
        result = roll("1d20")?;
    }
    assert_eq!(result.crit(), Crit::Failure);
    while [1, 20].contains(&result.total()) {
        result = roll("1d20")?;
    }
    assert_eq!(result.crit(), Crit::None);
    Ok(())
}

#[test]
fn test_literal() -> Result<(), Error<'static>> {
    assert_eq!(r("1")?, 1);
    assert_eq!(r("10000")?, 10000);
    assert_eq!(r("1.5")?, 1);
    assert_eq!(r("0.5")?, r(".5")?);
    assert_eq!(r("0.5")?, 0);
    Ok(())
}

#[test]
fn test_dice() -> Result<(), Error<'static>> {
    for _ in 0..1000 {
        assert_eq!(r("0d6")?, 0);
        assert!((1..=6).contains(&r("d6")?));
        assert!((1..=6).contains(&r("1d6")?));
        assert!((2..=12).contains(&r("2d6")?));
        assert_eq!(r("0d%")?, 0);
        assert!((1..=100).contains(&r("d%")?));
        assert!((1..=100).contains(&r("1d%")?));
        assert!((2..=200).contains(&r("2d%")?));
    }
    Ok(())
}

#[test]
fn test_set() -> Result<(), Error<'static>> {
    assert_eq!(r("(1)")?, 1);
    assert_eq!(r("(1,)")?, 1);
    assert_eq!(r("(1, 1)")?, 2);
    Ok(())
}

#[test]
fn test_unop() -> Result<(), Error<'static>> {
    assert_eq!(r("1")?, r("+1")?);
    assert_eq!(r("1")?, 1);
    assert_eq!(r("-1")?, -1);
    assert_eq!(r("--1")?, 1);
    assert_eq!(r("-+-++---+1")?, -1);
    Ok(())
}

#[test]
fn test_binop() -> Result<(), Error<'static>> {
    assert_eq!(r("2 + 2")?, 4);
    assert_eq!(r("2 - 2")?, 0);
    assert_eq!(r("2 * 5")?, 10);
    assert_eq!(r("15 / 2")?, 7);
    assert_eq!(r("15 // 2")?, 7);
    assert_eq!(r("13 % 2")?, 1);
    Ok(())
}

#[test]
fn test_binop_dice() -> Result<(), Error<'static>> {
    for _ in 0..1000 {
        assert!((3..=8).contains(&r("2 + 1d6")?));
        assert!((2..=12).contains(&r("2 * 1d6")?));
        assert!([60, 30, 20, 15, 12, 10].contains(&r("60 / 1d6")?));
        assert!([60, 30, 20, 15, 12, 10].contains(&r("60 // 1d6")?));
        assert!(r("1d100 % 10")? <= 9);
        assert!(r("1d% % 10")? <= 9);
    }
    Ok(())
}

#[test]
fn test_zero_division() {
    assert!(matches!(r("10 / 0"), Err(Error::ZeroDivision)));
    assert!(matches!(r("10 // 0"), Err(Error::ZeroDivision)));
    assert!(matches!(r("10 % 0"), Err(Error::ZeroDivision)));
}

#[test]
fn test_comparison() -> Result<(), Error<'static>> {
    assert_eq!(r("1 == 1")?, 1);
    assert_eq!(r("1 == 2")?, 0);
    assert_eq!(r("1 > 1")?, 0);
    assert_eq!(r("2 > 1")?, 1);
    assert_eq!(r("1 < 1")?, 0);
    assert_eq!(r("1 < 2")?, 1);
    assert_eq!(r("1 >= 1")?, 1);
    assert_eq!(r("1 >= 2")?, 0);
    assert_eq!(r("1 <= 1")?, 1);
    assert_eq!(r("2 <= 1")?, 0);
    assert_eq!(r("1 != 1")?, 0);
    assert_eq!(r("1 != 2")?, 1);
    Ok(())
}

#[test]
fn test_selectors() -> Result<(), Error<'static>> {
    assert_eq!(r("(1, 2, 3, 4, 5)k3")?, 3);
    assert_eq!(r("(1, 2, 3, 4, 5)k<3")?, 3);
    assert_eq!(r("(1, 2, 3, 4, 5)k>3")?, 9);
    assert_eq!(r("(1, 2, 3, 4, 5)kl2")?, 3);
    assert_eq!(r("(1, 2, 3, 4, 5)kh2")?, 9);
    assert_eq!(r("(1)k1")?, 1);
    assert_eq!(r("(1)k2")?, 0);
    Ok(())
}

#[test]
fn test_k() -> Result<(), Error<'static>> {
    assert_eq!(r("(1, 2, 3, 4, 5)k3")?, 3);
    assert_eq!(r("(1, 2, 3, 4, 5)k1k2")?, 3);
    assert_eq!(r("(1, 2, 3, 4, 5)kh1kl1")?, 6);
    Ok(())
}

#[test]
fn test_p() -> Result<(), Error<'static>> {
    assert_eq!(r("(1, 2, 3, 4, 5)p3")?, 12);
    assert_eq!(r("(1, 2, 3, 4, 5)p1p2")?, 12);
    assert_eq!(r("(1, 2, 3, 4, 5)ph1pl1")?, 9);
    Ok(())
}

#[test]
fn test_rr() -> Result<(), Error<'static>> {
    assert_eq!(r("1d20rr<20")?, 20);
    assert_eq!(r("1d20rr>1")?, 1);
    assert!(matches!(r("1d20rr<21"), Err(Error::RollLimitExceeded)));
    assert!(matches!(r("1d1rr1"), Err(Error::RollLimitExceeded)));
    Ok(())
}

#[test]
fn test_ro() -> Result<(), Error<'static>> {
    assert_eq!(r("1d1ro1")?, 1);
    assert!((1..=6).contains(&r("1d6rol1")?));
    Ok(())
}

#[test]
fn test_ra() -> Result<(), Error<'static>> {
    assert_eq!(r("1d1ra1")?, 2);
    assert!((2..=12).contains(&r("1d6ral1")?));
    Ok(())
}

#[test]
fn test_e() -> Result<(), Error<'static>> {
    assert_eq!(r("1d2e2")? % 2, 1);
    assert!(matches!(roll("1d20e<21"), Err(Error::RollLimitExceeded)));
    assert!(matches!(roll("1d1e1"), Err(Error::RollLimitExceeded)));
    Ok(())
}

#[test]
fn test_mi() -> Result<(), Error<'static>> {
    assert_eq!(r("10d6mi6")?, 60);
    assert_eq!(r("10d6mi10")?, 100);
    assert!((20..=60).contains(&r("10d6mi2")?));
    Ok(())
}

#[test]
fn test_ma() -> Result<(), Error<'static>> {
    assert_eq!(r("10d6ma1")?, 10);
    assert_eq!(r("10d6ma0")?, 0);
    assert!((10..=50).contains(&r("10d6ma5")?));
    Ok(())
}

#[test]
fn test_correct_results() -> Result<(), Error<'static>> {
    let mut result = roll("1 + 2 + 3")?;
    assert_eq!(result.total(), 6);
    assert_eq!(format!("{result}"), "1 + 2 + 3 = 6");
    let mut roll = result.expression().clone();
    let left = roll.left().unwrap_or_else(|| unreachable!("expression has one child")).clone();
    roll.set_left(BinOp::new(left, BinOpKind::Add, Literal::new(4.0).into())?);
    let dice::expr::NumberKind::Expression(e) = roll.kind() else { unreachable!() };
    result.set_expression(e.clone());
    assert_eq!(result.total(), 10);
    assert_eq!(format!("{result}"), "1 + 2 + 3 + 4 = 10");
    Ok(())
}
