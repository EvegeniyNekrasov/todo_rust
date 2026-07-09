use csv::WriterBuilder;
use serde::Deserialize;
use std::{
    collections::linked_list,
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
};

const FILE_NAME: &str = "./src/todos.csv";

#[derive(Debug, Clone, Deserialize)]
struct Todo {
    id: i64,
    title: String,
    finished: bool,
}

fn read_csv() -> Result<Vec<Todo>, Box<dyn Error>> {
    let file = File::open(FILE_NAME)?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut todos: Vec<Todo> = Vec::new();

    for result in rdr.deserialize::<Todo>() {
        let todo = result?;
        todos.push(todo);
    }

    Ok(todos)
}

fn get_from_user(mut value: String) -> i32 {
    io::stdin()
        .read_line(&mut value)
        .expect("failed to read line");

    let value = value.trim().parse().expect("Invalid input");
    return value;
}

fn add_todo() {
    let lista_todos = match read_csv() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error leyendo CSV , {}", e);
            Vec::new()
        }
    };

    println!("Insert title: ");

    let mut title = String::new();
    io::stdin().read_line(&mut title).expect("Invalid title");

    let title = title.trim().to_string();

    let id: i64 = if !lista_todos.is_empty() {
        lista_todos.last().unwrap().id + 1
    } else {
        1
    };

    let mut file: File = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(FILE_NAME)
        .expect("Could not open a file");

    let file_len = file.metadata().expect("Could not read metadata").len();
    if file_len > 0 {
        file.seek(SeekFrom::End(-1)).expect("Could not seek file");

        let mut last_byte = [0u8; 1];
        file.read_exact(&mut last_byte)
            .expect("Could not read last byte");

        if last_byte[0] != b'\n' {
            file.write_all(b"\n").expect("Could not write newline");
        }
    }

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

    wtr.write_record(&[id.to_string(), title, "false".to_string()])
        .expect("Could write to csv");
    wtr.flush().expect("Could save csv");

    println!("Saved to csv");
}

fn print_todos() {
    let lista_todos = match read_csv() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error leyendo CSV , {}", e);
            Vec::new()
        }
    };

    if lista_todos.len() == 0 {
        println!("No todo to show");
        return;
    }

    for i in 0..lista_todos.len() {
        println!(
            "ID: {}, Title: {}, Finished: {}",
            lista_todos[i].id,
            lista_todos[i].title,
            if lista_todos[i].finished {
                String::from("In process")
            } else {
                String::from("Finished")
            }
        )
    }
}

fn delete_todo() {
    let lista_todos = match read_csv() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error leyendo CSV , {}", e);
            Vec::new()
        }
    };

    if lista_todos.len() == 0 {
        println!("There is no todos to delete");
        return;
    }

    println!("Insert to delete ID");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Invalid id");
    let id_tod_delete: i64 = match input.trim().parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid ID");
            return;
        }
    };

    let original_len = lista_todos.len();
    let filtered_list: Vec<_> = lista_todos
        .into_iter()
        .filter(|todo| todo.id != id_tod_delete)
        .collect();

    if filtered_list.len() == original_len {
        println!("Todo width id {} not found", id_tod_delete);
        return;
    }

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(FILE_NAME)
        .expect("Could not open CSV");

    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

    wtr.write_record(&["id", "title", "finished"])
        .expect("Could not write heder");

    for todo in filtered_list {
        wtr.write_record(&[todo.id.to_string(), todo.title, todo.finished.to_string()])
            .expect("Could no save write todo");
    }

    wtr.flush().expect("Could not save CSV");
    println!("Todo deleted");
}

fn main() {
    println!("Welcome to rust todo!");
    println!("What you want todo:");

    let mut is_running = true;

    loop {
        if !is_running {
            break;
        }

        println!("1)    Show todos");
        println!("2)    Add todo");
        println!("3)    Delete todo");
        println!("4)    Update todo");
        println!("5)    Quit");

        let input = String::new();
        let input = get_from_user(input);

        match input {
            1 => print_todos(),
            2 => add_todo(),
            3 => delete_todo(),
            4 => println!("4"),
            5 => {
                println!("Finished");
                is_running = false;
            }
            _ => {
                print!("Unexpected value, insert number from 1 to 5");
                continue;
            }
        }
    }
}
