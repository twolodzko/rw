use crate::{
    Context, Parser,
    types::{Expr, Statement},
};
use anyhow::Result;

pub fn eval(statement: Statement, context: &mut Context) -> Result<()> {
    use Statement::*;
    match statement {
        LetEquals(lhs, rhs) => {
            if is_symmetric(&lhs, &rhs) {
                context.update_symmetries(&lhs);
            } else {
                context.insert_mapping(lhs.clone(), rhs.clone())?;
                context.insert_mapping(rhs, lhs)?;
            }
        }
        LetMap(lhs, rhs) => {
            if is_symmetric(&lhs, &rhs) {
                context.update_symmetries(&lhs);
            } else {
                context.insert_mapping(lhs, rhs)?;
            }
        }
        Rewrite(expr) => {
            expr.validate()?;
            let result = context.rewrite(expr);
            println!("{}", result);
        }
        Load(path) => eval_file(&path, context)?,
        Rules => println!("{}", context),
        // those statements are evaluated during parsing
        NewOp(_, _) => {}
    }
    Ok(())
}

pub fn eval_file(path: &str, context: &mut Context) -> Result<()> {
    let path = context.update_path(path)?;
    let parser = Parser::from_file(&path)?;
    let result = eval_all(parser, context);
    context.exit_path();
    result
}

fn eval_all<I>(iter: I, context: &mut Context) -> Result<()>
where
    I: Iterator<Item = Result<Statement>>,
{
    for statement in iter {
        eval(statement?, context)?;
    }
    Ok(())
}

fn is_symmetric(pattern: &Expr, template: &Expr) -> bool {
    use Expr::*;
    match (pattern, template) {
        // foo $A $B = foo $B $A
        (Cons(lo, la), Cons(ro, ra))
            if let Literal(lo) = lo.as_ref()
                && let Literal(ro) = ro.as_ref()
                && lo == ro
                && let Cons(ll, lr) = la.as_ref()
                && let Cons(rl, rr) = ra.as_ref() =>
        {
            ll.is_variable()
                && lr.is_variable()
                && rl.is_variable()
                && rr.is_variable()
                && ll != lr // this is not a symmetry
                && ll == rr
                && lr == rl
        }
        // $A * $B = $B * $A
        (Infix(lo, ll, lr), Infix(ro, rl, rr)) => {
            lo == ro
                && ll.is_variable()
                && lr.is_variable()
                && rl.is_variable()
                && rr.is_variable()
                && ll != lr // this is not a symmetry
                && ll == rr
                && lr == rl
        }
        _ => false,
    }
}

impl Expr {
    fn is_variable(&self) -> bool {
        matches!(self, Expr::Variable(_, _))
    }
}
