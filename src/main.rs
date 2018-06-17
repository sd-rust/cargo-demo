#[macro_use] extern crate error_chain;
extern crate serde_json;

use serde_json::Value;
use std::process::Command;

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Json(serde_json::Error);
    }
}

fn main() -> Result<()> {
    let cargo_path = env!("CARGO");
    println!("Path to cargo: {}", cargo_path);

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

    println!("{}", serde_json::to_string_pretty(&meta_data)?);

    Ok(())
}
