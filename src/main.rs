use anyhow::Result;
use rustyline::error::ReadlineError;
use rw::{Context, Parser, eval, eval_file};

fn main() -> Result<()> {
    let mut context = Context::default();

    let args = std::env::args();
    if args.len() == 1 {
        for r in Parser::from_stdin()? {
            match r {
                Ok(s) => {
                    if let Err(err) = eval(s, &mut context) {
                        println!("Error: {}", err)
                    }
                }
                Err(err)
                    if let Some(err) = err.downcast_ref::<ReadlineError>()
                        && let ReadlineError::Interrupted = err =>
                {
                    break;
                }
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            };
        }
    } else {
        for path in args.skip(1) {
            eval_file(&path, &mut context)?;
        }
    }
    Ok(())
}
