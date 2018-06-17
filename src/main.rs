#[macro_use] extern crate error_chain;
extern crate json;

use std::process::Command;

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Json(json::Error);
    }
}

fn main() -> Result<()> {
    let cargo_path = env!("CARGO");
    println!("Path to cargo: {}", cargo_path);

    let output = Command::new(cargo_path)
                    .arg("metadata")
                    .arg("--format-version").arg("1")
                    .output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    let meta_data = String::from_utf8(output.stdout)?;

    let meta_data = json::parse(&meta_data)?;

    println!("{}", json::stringify_pretty(meta_data , 4));

    Ok(())
}
