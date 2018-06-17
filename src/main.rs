#[macro_use] extern crate error_chain;
#[macro_use] extern crate human_panic;
extern crate serde_json;

use serde_json::Value;
use std::process::Command;
use ErrorKind::{JsonCastError, UserQuit};
use std::io;
use std::io::Write;
use std::env;

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Json(serde_json::Error);
    }

    errors {
        JsonCastError(key: &'static str) {
            description("invalid JSON cast")
            display("invalid cast used on JSON node: '{}'", key)
        }
        UserQuit {
            description("User quit")
            display("User quit")
        }
    }
}

fn get_example_names(meta_data: &Value) -> Result<Vec<Option<String>>> {
    let targets = &meta_data["packages"][0]["targets"];

    let target_names = targets.as_array().ok_or(JsonCastError("targets"))?.into_iter()
        .filter_map(|t| {
            if t["kind"].as_array().unwrap().into_iter()
                .map(|x| x.as_str())
                .any(|x| x == Some("example")) {
                Some(t["name"].as_str().unwrap().to_owned())
            }
            else {
                None
            }
        })
        .map(Some)
        .collect();

    Ok(target_names)
}

fn read_line(prompt: &str) -> Result<String> {
    print!("{}", prompt);

    io::stdout().flush()?;

    let mut line = String::new();

    io::stdin().read_line(&mut line)?;

    Ok(line.trim().into())
}

fn let_user_choose_example(examples: Vec<Option<String>>) -> Result<String> {

    println!("q) Quit");

    for (i, optional_example) in examples.iter().enumerate() {
        if let Some(example) = optional_example {
            println!("{}) {}", i+1, example);
        }
    }

    let mut examples = examples;

    loop {

        let choice = read_line("Enter choice: ");

        if let Ok(choice) = choice {
            if choice == "q" || choice == "Q" {
                return Err(UserQuit.into());
            }

            if let Ok(index) = choice.parse::<usize>() {
                if let Some(example) = examples[index-1].take() {
                    return Ok(example);
                }
            }
            else {
                println!("[cargo demo] Bad choice, try again.");
            }
        }
    }
}

fn build_and_run_example(example: &str, extra_args: &[String]) -> Result<()> {
    let cargo_path = env!("CARGO");

    let status = Command::new(cargo_path)
        .arg("run")
        .args(extra_args)
        .arg("--example").arg(example)
        .status()?;

    if !status.success() {
        match status.code() {
            Some(exit_code) => eprintln!("[cargo demo] Bad cargo exit code: {}", exit_code),
            None => eprintln!("[cargo demo] Process terminated by signal"),
        }
    }

    Ok(())
}

fn get_crate_metadata() -> Result<Value> {
    let cargo_path = env!("CARGO");

    let output = Command::new(cargo_path)
        .arg("metadata")
        .arg("--format-version").arg("1")
        .arg("--no-deps")
        .output()?;


    let meta_data = String::from_utf8(output.stdout)?;

    Ok(serde_json::from_str(&meta_data)?)
}

fn main() -> Result<()> {
    setup_panic!();

    let mut extra_args: Vec<String> = env::args().skip(2).collect();

    let mut run_all_examples = false;

    if let Some(arg) = extra_args.get(0) {
        if arg.to_lowercase() == "all" {
            run_all_examples = true;
        }
    }

    if run_all_examples {
        extra_args.remove(0);
    }


    let extra_args = extra_args;

    let meta_data = get_crate_metadata();

    if let Err(ref e) = &meta_data {
        if let ErrorKind::Io(_) = e.kind() {
            eprintln!("[cargo demo] Command executed with failing error code");
        }
    }

    let meta_data = meta_data?;

    let examples = get_example_names(&meta_data)?;

    if !run_all_examples {
        match let_user_choose_example(examples) {
            Ok(ref example_name) => build_and_run_example(example_name, &extra_args),
            Err(e) => match e.kind() {
                ErrorKind::UserQuit => {
                    eprintln!("[cargo demo] Bye!");
                    Ok(())
                }
                _ => Err(e)
            }
        }
    }
    else {
        for optional_example in &examples {
            if let Some(example) = optional_example {
                println!("[cargo demo] Running example: {}", example);
                build_and_run_example(example, &extra_args)?;
            }
        }
        Ok(())
    }
}
