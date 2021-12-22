use std::{env, process};

fn run_prompt() -> anyhow::Result<()> {
    todo!()
}

fn run_file(_path: &str) -> anyhow::Result<()> {
    todo!()
}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]),
        _ => {
            println!("Usage: lox [script]");
            process::exit(1);
        }
    }
}
