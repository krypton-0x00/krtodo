use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;

pub struct App {
    pub items: usize,
    pub db: String,
}
#[derive(Debug, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub is_completed: bool,
}
fn count_todos(db_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let mut rdr = Reader::from_path(db_path).expect("Error While Reading Db");
    let count = rdr.records().count();
    Ok(count)
}
impl Default for App {
    fn default() -> Self {
        let path_to_db: String = "/home/nesu/todos.csv".to_string();
        let mut todos: usize = 0;

        if Path::new(&path_to_db).exists() {
            match count_todos(&path_to_db) {
                Ok(count) => todos = count,
                Err(err) => eprintln!("Error reading the db: {}", err),
            }
            println!("File exists.")
        } else {
            let _ = File::create(&path_to_db);
            let mut wtr = Writer::from_path(&path_to_db).unwrap();
            wtr.write_record(["id", "title", "is_completed"]).unwrap();
            println!("File created.")
        }

        Self {
            items: todos,
            db: path_to_db.to_string(),
        }
    }
}
impl App {
    pub fn read_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = Reader::from_path(&self.db).unwrap();
        println!("DB readed.");
        for result in rdr.deserialize() {
            let todo: Todo = result?;
            println!("{:?}", todo);
        }
        Ok(())
    }
    pub fn write_db(&self, todo: Todo) -> Result<(), Box<dyn std::error::Error>> {
        let file = OpenOptions::new().append(true).open(&self.db)?;

        let mut database = Writer::from_writer(file);

        database.write_record([
            &(&self.items + 1).to_string(),
            &todo.title.clone(),
            &todo.is_completed.to_string(),
        ])?;
        database.flush()?;
        Ok(())
    }
    pub fn update_todo(&mut self, id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(&self.db)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(contents.as_bytes());
        let mut records = vec![];

        for result in rdr.records() {
            let record = result?;
            let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            let record_id: u32 = fields[0].parse()?;

            if record_id == id {
                let current_status: bool = fields[2].parse()?;
                fields[2] = (!current_status).to_string();
            }

            records.push(fields);
        }

        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(File::create(&self.db)?);

        for record in records {
            wtr.write_record(&record)?;
        }

        wtr.flush()?;
        println!("Todo with ID {} updated successfully!", id);
        Ok(())
    }
}
