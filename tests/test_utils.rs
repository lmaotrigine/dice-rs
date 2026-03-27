use dice::{
    Advantage, Error, Formatter, SimpleFormatter,
    ast::{DiceExpr, Node},
    expr::{Literal, Number, NumberKind},
    parse, roll, roll_expr_with_default_rng, utils,
};

#[test]
fn test_leftmost() -> Result<(), Error<'static>> {
    let tree = parse("1d20 + 4d6 + 3", false)?;
    assert_eq!(format!("{}", utils::leftmost(&Node::from(tree.clone()))), "1d20");
    let res = roll_expr_with_default_rng(tree, Advantage::None)?;
    let expr = res.expression();
    assert!(SimpleFormatter.format(utils::leftmost(expr)).starts_with("1d20 "));
    Ok(())
}

#[test]
fn test_rightmost() -> Result<(), Error<'static>> {
    let tree = parse("1d20 + 4d6 + 3", false)?;
    assert_eq!(format!("{}", utils::rightmost(&Node::from(tree.clone()))), "3");
    let res = roll_expr_with_default_rng(tree, Advantage::None)?;
    let expr = res.expression();
    assert_eq!(SimpleFormatter.format(utils::rightmost(expr)), "3");
    Ok(())
}

#[test]
fn test_dfs() -> Result<(), Error<'static>> {
    let mixed = roll("-1d8 + 4 - (3, 1d4)kh1")?;
    let result =
        utils::dfs(mixed.expression(), |n| matches!(n.kind(), NumberKind::Dice(d) if d.num() == 1 && d.sides() == 4));
    assert!(result.is_some());
    assert!(SimpleFormatter.format(result.unwrap()).starts_with("1d4 "));
    Ok(())
}

#[test]
fn test_ast_map() -> Result<(), Error<'static>> {
    let tree = parse("1d20 + 4d6 + 3", false)?.into();
    let mapper = |node| {
        if let Node::DiceExpr(d) = node { Node::DiceExpr(DiceExpr::new(d.num() * 2, d.sides())) } else { node.clone() }
    };
    let mapped = utils::map_tree(&tree, mapper);
    assert_eq!(format!("{mapped}"), "2d20 + 8d6 + 3");
    assert_eq!(format!("{tree}"), "1d20 + 4d6 + 3");
    Ok(())
}

#[test]
fn test_expr_map() -> Result<(), Error<'static>> {
    let res = roll("1 + 2 + 3")?;
    let expr = res.expression();
    let mapper = |node: Number| {
        if let NumberKind::Literal(l) = node.kind() {
            let mut values = l.values().to_vec();
            if let Some(v) = values.last_mut() {
                *v *= 2.0;
            }
            let mut new = Literal::new(values[0]);
            for v in values.iter().skip(1) {
                new.update(*v);
            }
            new.into()
        } else {
            node.clone()
        }
    };
    let mapped = utils::map_tree(expr, mapper);
    assert_eq!(SimpleFormatter.format(&mapped), "2 + 4 + 6 = 12");
    assert_eq!(mapped.total(), 12.0);
    assert_eq!(SimpleFormatter.format(expr), "1 + 2 + 3 = 6");
    Ok(())
}

#[test]
fn test_adv_clone() -> Result<(), Error<'static>> {
    for expr in ["1d20", "1d20+1", "1d20-1d4"] {
        let tree: Node = parse(expr, false)?.into();
        let orig = tree.clone();
        let adv_tree = utils::ast_with_adv(tree.clone(), Advantage::Advantage);
        assert!(format!("{adv_tree}").starts_with("2d20kh1"));
        assert_ne!(format!("{adv_tree}"), format!("{orig}"));
        let adv_tree = utils::ast_with_adv(tree.clone(), Advantage::Disadvantage);
        assert!(format!("{adv_tree}").starts_with("2d20kl1"));
        assert_ne!(format!("{adv_tree}"), format!("{orig}"));
        let adv_tree = utils::ast_with_adv(tree.clone(), Advantage::None);
        assert!(format!("{adv_tree}").starts_with("1d20"));
        assert_eq!(format!("{adv_tree}"), format!("{orig}"));
    }
    Ok(())
}

#[test]
fn test_inapplicable_adv_clone() -> Result<(), Error<'static>> {
    for expr in ["1", "1d6", "1+1"] {
        let tree: Node = parse(expr, false)?.into();
        let orig = tree.clone();
        let adv_tree = utils::ast_with_adv(tree, Advantage::Advantage);
        assert_eq!(format!("{adv_tree}"), format!("{orig}"));
    }
    Ok(())
}
