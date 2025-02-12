use krtodo::App;
use krtodo::Todo;
fn main() {
    let app = App::default();
    let t1 = Todo {
        id: 1,
        title: "Go to Gym".to_string(),
        is_completed: false,
    };
    app.write_db(t1).unwrap();
    app.read_db().unwrap();
}
