#[macro_use] extern crate error_chain;
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

fn main() -> Result<()> {
    let cargo_path = env!("CARGO");

    let output = Command::new(cargo_path)
                    .arg("metadata")
                    .arg("--format-version").arg("1")
                    .arg("--no-deps")
                    .output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    let meta_data = String::from_utf8(output.stdout)?;

    let meta_data: Value = serde_json::from_str(&meta_data)?;

    let examples = get_example_names(&meta_data)?;

    for example in &examples {
        println!("{}", example);
    }

    Ok(())
}
