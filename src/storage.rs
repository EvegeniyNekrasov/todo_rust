use crate::model::Todo;
use csv::{ReaderBuilder, WriterBuilder};
use std::{error::Error, fs, path::Path};

pub type StorageResult<T> = Result<T, Box<dyn Error>>;

pub fn load_todos(path: &Path) -> StorageResult<Vec<Todo>> {
    create_parent_directory(path)?;

    if !path.exists() || fs::metadata(path)?.len() == 0 {
        save_todos(path, &[])?;
        return Ok(Vec::new());
    }

    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;

    let todos = reader
        .deserialize::<Todo>()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(todos)
}

pub fn save_todos(path: &Path, todos: &[Todo]) -> StorageResult<()> {
    create_parent_directory(path)?;

    let mut writer = WriterBuilder::new().has_headers(false).from_path(path)?;

    writer.write_record(["id", "title", "finished"])?;

    for todo in todos {
        writer.serialize(todo)?;
    }

    writer.flush()?;

    Ok(())
}

fn create_parent_directory(path: &Path) -> StorageResult<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
