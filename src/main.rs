#[macro_use] extern crate error_chain;
#[macro_use] extern crate human_panic;
extern crate serde_json;

use serde_json::Value;
use std::process::Command;
use ErrorKind::JsonCastError;
use std::io;
use std::io::Write;

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
    }
}

fn get_example_names(meta_data: &Value) -> Result<Vec<String>> {
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

fn list_examples(meta_data: &Value) -> Result<()> {
    let examples = get_example_names(meta_data)?;

    for (i, example) in examples.iter().enumerate() {
        println!("{}) {}", i+1, example);
    }

    println!("q) Quit");

    loop {

        let choice = read_line("Enter choice: ");

        if let Ok(choice) = choice {
            if choice == "q" || choice == "Q" {
                println!("Bye.");
                break;
            }

            if let Ok(index) = choice.parse::<usize>() {
                if let Some(example) = examples.get(index-1) {
                    build_and_run_example(example)?;
                    break;
                }
            }
            else {
                println!("Bad choice, try again.");
            }
        }
    }

    Ok(())
}

fn build_and_run_example(example: &str) -> Result<()> {
    let cargo_path = env!("CARGO");

    let status = Command::new(cargo_path)
        .arg("run")
        .arg("--release")
        .arg("--example").arg(example)
        .status()?;

    if !status.success() {
        match status.code() {
            Some(exit_code) => eprintln!("Bad cargo exit code: {}", exit_code),
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

    let meta_data = get_crate_metadata();

    if let Err(ref e) = &meta_data {
        if let ErrorKind::Io(_) = e.kind() {
            eprintln!("Command executed with failing error code");
        }
    }

    let meta_data = meta_data?;

    list_examples(&meta_data)
}
