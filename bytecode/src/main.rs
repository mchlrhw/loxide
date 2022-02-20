use lox_bytecode::vm::Vm;
use std::{env, io::Write, process};

fn repl(vm: &mut Vm) -> anyhow::Result<()> {
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.is_empty() {
            break;
        }

        let _ = vm.interpret(&line);
    }

    Ok(())
}

fn run_file(path: &str, vm: &mut Vm) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;

    vm.interpret(&source)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut vm = Vm::new();

    match args.len() {
        0 => repl(&mut vm),
        1 => run_file(&args[0], &mut vm),
        _ => {
            println!("Usage: lox [script]");
            process::exit(1);
        }
    }
}
