use lox::{interpreter::Interpreter, parser::Parser, scanner::Scanner};
use std::{env, io::Write, process};

fn run(source: &str) -> anyhow::Result<()> {
    let mut scanner = Scanner::new(source);

    let tokens = scanner.scan();
    let mut parser = Parser::new(tokens);

    if let Ok(statements) = parser.parse() {
        Interpreter::new().interpret(statements);
    }

    Ok(())
}

fn run_prompt() -> anyhow::Result<()> {
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.is_empty() {
            break;
        }

        run(&line)?;
    }

    Ok(())
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
