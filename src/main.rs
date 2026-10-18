use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use rustyline::error::ReadlineError;
use rw::{Parser, WORKING_DIRECTORY, eval, eval_file};

fn main() -> Result<()> {
    let mut context = rw::Context::default();

    let args = std::env::args();
    if args.len() == 1 {
        #[allow(static_mut_refs)]
        unsafe {
            WORKING_DIRECTORY.push(current_dir()?);
        }

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
            let path = PathBuf::from(path);
            #[allow(static_mut_refs)]
            unsafe {
                WORKING_DIRECTORY.push(path.parent().unwrap().into());
            }
            eval_file(&path, &mut context)?;
        }
    }
    Ok(())
}
