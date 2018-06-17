#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate human_panic;
extern crate serde_json;
extern crate cursive;

use serde_json::Value;
use std::process::Command;
use ErrorKind::JsonBadCast;
use cursive::views::SelectView;
use cursive::align::HAlign;
use cursive::views::OnEventView;
use cursive::event::EventResult;
use cursive::views::Dialog;
use cursive::Cursive;
use cursive::views::TextView;
use cursive::traits::Boxable;

error_chain!{
    foreign_links {
        Io(std::io::Error);
        Utf8(std::string::FromUtf8Error);
        Json(serde_json::Error);
    }

    errors {
        JsonBadCast(key: &'static str) {
            description("invalid JSON cast")
            display("invalid cast used on JSON node: '{}'", key)
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

// Let's put the callback in a separate function to keep it clean,
// but it's not required.
fn show_next_window(siv: &mut Cursive, city: &str) {
    siv.pop_layer();
    let text = format!("Selected: {}.", city);
    siv.add_layer(
        Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()),
    );
}

fn list_examples(meta_data: &Value) -> Result<()>{
    let examples = get_example_names(meta_data)?;

    for example in &examples {
        println!("{}", example);
    }

    let mut select = SelectView::new().h_align(HAlign::Center);

    select.add_all_str(examples);

    // Sets the callback for when "Enter" is pressed.
    select.set_on_submit(show_next_window);

    // Let's override the `j` and `k` keys for navigation
    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('j', |s| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        });

    let mut siv = Cursive::default();

    // Let's add a BoxView to keep the list at a reasonable size
    // (it can scroll anyway).
    siv.add_layer(
        Dialog::around(select.fixed_size((20, 10)))
            .title("Run example ..."),
    );

    siv.run();

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
