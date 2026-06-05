//! The `lox` command-line interface.
//!
//! With a script path it runs that file; with `-` (or piped stdin) it runs the
//! program read from standard input; with no arguments and a terminal attached
//! it starts an interactive REPL.

use std::io::{IsTerminal, Read, Write};
use std::process::ExitCode;

use lox_interpreter::{Interpreter, Value, run};

const USAGE: &str = "\
Usage: lox [script]

  script        path to a .lox file ('-' reads the program from stdin)
  -h, --help    print this help and exit

With no script, lox starts a REPL (or runs stdin if it is piped).";

fn main() -> ExitCode {
    let mut script: Option<String> = None;
    let mut options_done = false;

    for arg in std::env::args().skip(1) {
        if !options_done {
            match arg.as_str() {
                "--" => {
                    options_done = true;
                    continue;
                }
                "-h" | "--help" => {
                    println!("{USAGE}");
                    return ExitCode::SUCCESS;
                }
                // An unknown flag — the lone "-" stdin marker is not a flag.
                other if other.starts_with('-') && other != "-" => {
                    eprintln!("error: unknown option '{arg}'\n\n{USAGE}");
                    return ExitCode::from(64); // EX_USAGE
                }
                _ => {}
            }
        }

        if script.is_some() {
            eprintln!("error: unexpected argument '{arg}'\n\n{USAGE}");
            return ExitCode::from(64); // EX_USAGE
        }
        script = Some(arg);
    }

    match script.as_deref() {
        Some("-") => run_stdin(),
        Some(path) => run_file(path),
        None if std::io::stdin().is_terminal() => run_repl(),
        None => run_stdin(),
    }
}

/// Evaluate `source` to completion, reporting a runtime error to stderr.
fn run_source(source: &str) -> ExitCode {
    let interpreter = Interpreter::new(std::io::stdout());
    match run(source, &interpreter) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(70) // EX_SOFTWARE
        }
    }
}

fn run_file(path: &str) -> ExitCode {
    match std::fs::read_to_string(path) {
        Ok(source) => run_source(&source),
        Err(error) => {
            eprintln!("error: could not read '{path}': {error}");
            ExitCode::from(66) // EX_NOINPUT
        }
    }
}

fn run_stdin() -> ExitCode {
    let mut source = String::new();
    if let Err(error) = std::io::stdin().read_to_string(&mut source) {
        eprintln!("error: could not read stdin: {error}");
        return ExitCode::from(66); // EX_NOINPUT
    }
    run_source(&source)
}

/// Read-eval-print loop. A single interpreter is reused across lines so that
/// variables declared earlier remain in scope.
fn run_repl() -> ExitCode {
    let interpreter = Interpreter::new(std::io::stdout());
    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        if std::io::stdout().flush().is_err() {
            break;
        }

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF (Ctrl-D)
            Ok(_) => {}
            Err(error) => {
                eprintln!("error: {error}");
                break;
            }
        }

        match run(&line, &interpreter) {
            // Statements evaluate to Nil; only echo a meaningful expression value.
            Ok(Value::Nil) => {}
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("{error}"),
        }
    }

    ExitCode::SUCCESS
}
