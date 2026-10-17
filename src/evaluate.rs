use crate::{
    Parser,
    mapping::Mapping,
    operators::Op,
    simplify::Rules,
    types::{Expr, Statement},
};
use anyhow::{Result, bail};

pub fn eval(statement: Statement, rules: &mut Rules) -> Result<()> {
    use Statement::*;
    match statement {
        Let(expr) => {
            let reverse = matches!(expr, Expr::Infix(Op::Equals, _, _));

            let m = Mapping::try_from(expr)?;
            rules.insert_mapping(m.clone())?;

            if reverse {
                let m = m.reversed()?;
                rules.insert_mapping(m)?
            }
        }
        LetMap(map) => rules.insert_mapping(map)?,
        Eval(expr) => {
            expr.validate()?;
            let result = rules.simplify(expr);
            println!("{}", result);
        }
        Assert(expr) => {
            expr.validate()?;
            let ok = rules.simplify(expr.clone());
            if !matches!(ok, Expr::Bool(true)) {
                bail!("{} is not true", expr)
            }
        }
        EvalTo(key, expr) => {
            let val = rules.simplify(expr);
            let m = Mapping::new(Expr::Literal(key), val)?;
            rules.insert_mapping(m)?;
        }
        Load(path) => eval_file(&path, rules)?,
        // those statements are evaluated during parsing
        NewInfix(_, _) | NewPrefix(_, _) => {}
    }
    Ok(())
}

pub fn eval_file(path: &str, rules: &mut Rules) -> Result<()> {
    let parser = Parser::from_file(&path)?;
    for statement in parser {
        eval(statement?, rules)?;
    }
    Ok(())
}
