use crate::{Parser, WORKING_DIRECTORY, mapping::Mapping, rewrite::Context, types::Statement};
use anyhow::Result;
use relative_path::RelativePath;
use std::path::PathBuf;

pub fn eval(statement: Statement, context: &mut Context) -> Result<()> {
    use Statement::*;
    match statement {
        LetEquals(lhs, rhs) => {
            context.insert_mapping(Mapping::new(lhs.clone(), rhs.clone())?)?;
            context.insert_mapping(Mapping::new(rhs, lhs)?)?;
        }
        LetMap(lhs, rhs) => context.insert_mapping(Mapping::new(lhs, rhs)?)?,
        Rewrite(expr) => {
            expr.validate()?;
            let result = context.rewrite(expr);
            println!("{}", result);
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
        Rules => println!("{}", context),
        // those statements are evaluated during parsing
        NewOp(_, _) => {}
    }
    Ok(())
}

pub fn eval_file(path: &PathBuf, context: &mut Context) -> Result<()> {
    let parser = Parser::from_file(path)?;
    for statement in parser {
        eval(statement?, context)?;
    }
    Ok(())
}
