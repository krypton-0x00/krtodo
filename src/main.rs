use krtodo::start_tui;

fn main() {
    let db_path = "/home/nesu/todos.csv";
    if let Err(err) = start_tui(db_path) {
        eprintln!("Error running TUI: {}", err);
    }
}
