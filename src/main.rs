use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use csv::{Writer, WriterBuilder};
use serde::Deserialize;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write, stdout},
};

const FILE_NAME: &str = "./src/todos.csv";

fn clear_console() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
}

fn pause() {
    println!("\nPress ENTER to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

#[derive(Debug, Clone, Deserialize)]
struct Todo {
    id: i64,
    title: String,
    finished: bool,
}

fn save_todos(todos: &[Todo]) -> Result<(), Box<dyn Error>> {
    let file: File = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(FILE_NAME)?;

    let mut wtr: Writer<File> = WriterBuilder::new().has_headers(true).from_writer(file);

    wtr.write_record(&["id", "title", "finished"])?;

    for todo in todos {
        wtr.write_record(&[
            todo.id.to_string(),
            todo.title.clone(),
            todo.finished.to_string(),
        ])?;
    }

    wtr.flush()?;

    Ok(())
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
                String::from("Finished")
            } else {
                String::from("In process")
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

    match save_todos(&filtered_list) {
        Ok(_) => println!("Todo delted"),
        Err(e) => println!("Could not delete todo: {}", e),
    }
}

fn update_todo() {
    let mut todos_list = match read_csv() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error reading CSV, {}", e);
            return;
        }
    };

    if todos_list.is_empty() {
        println!("There are no todos to update");
        return;
    }

    println!("Insert ID to update:");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Invalid id");

    let id_to_update: i64 = match input.trim().parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid ID");
            return;
        }
    };

    let todo_to_update = todos_list.iter_mut().find(|todo| todo.id == id_to_update);

    let todo = match todo_to_update {
        Some(todo) => todo,
        None => {
            println!("Todo with id {} not found", id_to_update);
            return;
        }
    };

    println!("Current title: {}", todo.title);
    println!("Insert new title, or press Enter to keep current title:");

    let mut new_title = String::new();
    io::stdin()
        .read_line(&mut new_title)
        .expect("Invalid title");

    let new_title = new_title.trim();

    if !new_title.is_empty() {
        todo.title = new_title.to_string();
    }

    println!("Current finished: {}", todo.finished);
    println!("Insert new finished value: true / false, or press Enter to keep current value:");

    let mut new_finished = String::new();
    io::stdin()
        .read_line(&mut new_finished)
        .expect("Invalid finished value");

    let new_finished = new_finished.trim().to_lowercase();

    if !new_finished.is_empty() {
        match new_finished.as_str() {
            "true" | "t" | "1" | "yes" | "y" | "si" | "s" => {
                todo.finished = true;
            }
            "false" | "f" | "0" | "no" | "n" => {
                todo.finished = false;
            }
            _ => {
                println!("Invalid finished value");
                return;
            }
        }
    }

    match save_todos(&todos_list) {
        Ok(_) => println!("Todo updated"),
        Err(e) => println!("Could not update todo: {}", e),
    }
}

fn main() {
    loop {
        clear_console();

        println!("Welcome to rust todo!");
        println!("What you want todo:\n");

        println!("1)\tShow todos");
        println!("2)\tAdd todo");
        println!("3)\tDelete todo");
        println!("4)\tUpdate todo");
        println!("5)\tQuit");

        let input = String::new();
        let input = get_from_user(input);

        match input {
            1 => print_todos(),
            2 => add_todo(),
            3 => delete_todo(),
            4 => update_todo(),
            5 => break,
            _ => print!("Invalid option..."),
        }

        pause();
    }
}
