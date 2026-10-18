use std::path::PathBuf;

use crate::{
    Parser, WORKING_DIRECTORY,
    mapping::Mapping,
    operators::Op,
    simplify::Context,
    types::{Expr, Statement},
};
use anyhow::{Result, bail};
use relative_path::RelativePath;

pub fn eval(statement: Statement, context: &mut Context) -> Result<()> {
    use Statement::*;
    match statement {
        Let(expr) => {
            let reverse = matches!(expr, Expr::Infix(Op::Equals, _, _));

            let m = Mapping::try_from(expr)?;
            context.insert_mapping(m.clone())?;

            if reverse {
                let m = m.reversed()?;
                context.insert_mapping(m)?
            }
        }
        LetMap(map) => context.insert_mapping(map)?,
        Eval(expr) => {
            expr.validate()?;
            let result = context.simplify(expr);
            println!("{}", result);
        }
        Assert(expr) => {
            expr.validate()?;
            let ok = context.simplify(expr.clone());
            if !matches!(ok, Expr::Bool(true)) {
                bail!("{} is not true", expr)
            }
        }
        EvalTo(key, expr) => {
            let val = context.simplify(expr);
            let m = Mapping::new(Expr::Literal(key), val)?;
            context.insert_mapping(m)?;
        }
        Load(path) => {
            #[allow(static_mut_refs)]
            unsafe {
                let parent = WORKING_DIRECTORY.last().unwrap();
                let path = RelativePath::new(&path).to_path(parent);
                WORKING_DIRECTORY.push(path.parent().unwrap().into());
                eval_file(&path, context)?;
                WORKING_DIRECTORY.pop();
            }
        }
        // those statements are evaluated during parsing
        NewInfix(_, _) | NewPrefix(_, _) => {}
    }
    Ok(())
}

pub fn eval_file(path: &PathBuf, context: &mut Context) -> Result<()> {
    let parser = Parser::from_file(&path)?;
    for statement in parser {
        eval(statement?, context)?;
    }
    Ok(())
}
