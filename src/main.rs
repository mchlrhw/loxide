use lox::{interpreter::Interpreter, parser::Parser, resolver::Resolver, scanner::Scanner};
use std::{env, io::Write, process};

fn run(interpreter: &mut Interpreter, source: &str) {
    let mut scanner = Scanner::new(source);

    let tokens = scanner.scan();
    let mut parser = Parser::new(tokens);

    if let Ok(statements) = parser.parse() {
        let mut resolver = Resolver::new(interpreter);
        resolver.resolve_statements(statements.clone());
        if resolver.had_error() {
            return;
        }

        interpreter.interpret(statements);
    }
}

fn run_prompt() -> anyhow::Result<()> {
    let mut interpreter = Interpreter::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.is_empty() {
            break;
        }

        run(&mut interpreter, &line);
    }

    Ok(())
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;
    let mut interpreter = Interpreter::new();

    run(&mut interpreter, &source);

    Ok(())
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
