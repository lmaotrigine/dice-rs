use dice::{Advantage, Error, RollResult, parse, roll, roll_expr_with_default_rng, roll_with_comments};

fn roll_expr(input: &'static str, allow_comments: bool) -> Result<RollResult, Error<'static>> {
    if allow_comments { roll_with_comments(input) } else { roll(input) }
}

fn roll_adv(input: &'static str, adv: Advantage) -> Result<RollResult, Error<'static>> {
    let expr = parse(input, false)?;
    roll_expr_with_default_rng(expr, adv)
}

#[test]
fn test_comments() -> Result<(), Error<'static>> {
    let r = roll_expr("1d20 foo bar", true)?;
    assert!((1..=20).contains(&r.total()));
    assert_eq!(r.comment(), Some("foo bar"));
    assert!(matches!(roll_expr("1d20 foo bar", false), Err(Error::Syntax(_))));
    Ok(())
}

#[test]
fn test_conflicting_comments() -> Result<(), Error<'static>> {
    let r = roll_expr("1d20 keep something", true)?;
    assert!((1..=20).contains(&r.total()));
    assert_eq!(r.comment(), Some("keep something"));
    assert!(matches!(roll_expr("1d20 keep something", false), Err(Error::Syntax(_))));
    let r = roll_expr("1d20 damage", true)?;
    assert!((1..=20).contains(&r.total()));
    assert_eq!(r.comment(), Some("damage"));
    assert!(matches!(roll_expr("1d20 damage", false), Err(Error::Syntax(_))));
    let r = roll_expr("1d20 **bold**", true)?;
    assert!((1..=20).contains(&r.total()));
    assert_eq!(r.comment(), Some("**bold**"));
    assert!(matches!(roll_expr("1d20 **bold**", false), Err(Error::Syntax(_))));
    let r = roll_expr("1d20 please save me from this parsing weirdness", true)?;
    assert!((1..=20).contains(&r.total()));
    assert_eq!(r.comment(), Some("please save me from this parsing weirdness"));
    assert!(matches!(roll_expr("1d20 please save me from this parsing weirdness", false), Err(Error::Syntax(_))));
    Ok(())
}

#[test]
fn test_advantage() -> Result<(), Error<'static>> {
    let r = roll_adv("1d20", Advantage::Advantage)?;
    assert!((1..=20).contains(&r.total()));
    assert!(format!("{r}").starts_with("2d20kh1 "));
    let r = roll_adv("1d20", Advantage::Disadvantage)?;
    assert!((1..=20).contains(&r.total()));
    assert!(format!("{r}").starts_with("2d20kl1 "));
    let r = roll_adv("1d20", Advantage::None)?;
    assert!((1..=20).contains(&r.total()));
    assert!(format!("{r}").starts_with("1d20 "));
    let r = roll_adv("1d6", Advantage::Advantage)?;
    assert!((1..=6).contains(&r.total()));
    assert!(format!("{r}").starts_with("1d6 "));
    let r = roll_adv("1d6", Advantage::Disadvantage)?;
    assert!((1..=6).contains(&r.total()));
    assert!(format!("{r}").starts_with("1d6 "));
    Ok(())
}

#[test]
fn test_ast() -> Result<(), Error<'static>> {
    let ast = parse("1d20", false)?;
    let r = roll_expr_with_default_rng(ast, Advantage::None)?;
    assert!((1..=20).contains(&r.total()));
    assert!(format!("{r}").starts_with("1d20 "));
    Ok(())
}
