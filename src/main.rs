#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate human_panic;
extern crate serde_json;

use serde_json::Value;
use std::process::Command;
use ErrorKind::JsonBadCast;

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Json(serde_json::Error);
    }

    errors {
        JsonBadCast(key: &'static str) {
            description("invalid JSON cast")
            display("invalid cast used on JSON key: '{}'", key)
        }
    }
}

fn get_example_names(meta_data: &Value) -> Result<Vec<String>> {
    let targets = &meta_data["packages"][0]["targets"];

    let target_names = targets.as_array().ok_or(JsonBadCast("targets"))?.into_iter()
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

fn list_examples(meta_data: &Value) -> Result<()>{
    let examples = get_example_names(meta_data)?;

    for example in &examples {
        println!("{}", example);
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
