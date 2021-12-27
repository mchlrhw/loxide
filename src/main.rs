use std::{env, io::Write, process};

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

#[derive(Debug)]
struct Token;

struct Scanner;

impl Scanner {
    fn new(input: &str) -> Self {
        Self
    }

    fn scan(&mut self) -> Vec<Token> {
        todo!();
    }
}

fn run(input: &str) -> anyhow::Result<()> {
    let mut scanner = Scanner::new(input);

    for token in scanner.scan() {
        println!("{:?}", token);
    }

    Ok(())
}

fn run_prompt() -> anyhow::Result<()> {
    let mut line = String::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

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
