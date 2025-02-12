use krtodo::start_tui;
use std::process::Command;

fn main() {
    let cmd = Command::new("sh")
        .arg("-c")
        .arg("whoami")
        .output()
        .expect("Error while getting user.");

    let mut user = String::from_utf8(cmd.stdout).unwrap();
    user.pop();
    let db_path = format!("/home/{}/todos.csv", user);
    if let Err(err) = start_tui(&db_path) {
        eprintln!("Error running TUI: {}", err);
    }
}
