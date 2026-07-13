use crate::{model::Todo, storage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;
use std::{error::Error, path::PathBuf};

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    Add,
    Rename,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing(EditorAction),
    ConfirmDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

#[derive(Debug)]
pub struct Status {
    kind: StatusKind,
    message: String,
}

impl Status {
    pub fn kind(&self) -> StatusKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub struct App {
    path: PathBuf,
    todos: Vec<Todo>,
    table_state: TableState,

    mode: Mode,
    input: String,
    status: Status,

    should_exit: bool,
}

impl App {
    pub fn new(path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();
        let todos = storage::load_todos(&path)?;

        let mut table_state = TableState::default();

        if !todos.is_empty() {
            table_state.select(Some(0));
        }

        Ok(Self {
            path,
            todos,
            table_state,
            mode: Mode::Normal,
            input: String::new(),
            status: Status {
                kind: StatusKind::Info,
                message: "Ready".to_string(),
            },
            should_exit: false,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_exit = true;
            return;
        }

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Editing(action) => self.handle_editor_key(key, action),
            Mode::ConfirmDelete => self.handle_delete_confirmation(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_exit = true;
            }

            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
            }

            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
            }

            KeyCode::Home => {
                self.select_first();
            }

            KeyCode::End => {
                self.select_last();
            }

            KeyCode::Char('a') => {
                self.start_add();
            }

            KeyCode::Char('e') | KeyCode::Enter => {
                self.start_rename();
            }

            KeyCode::Char(' ') => {
                self.toggle_selected();
            }

            KeyCode::Char('d') | KeyCode::Delete => {
                self.start_delete();
            }

            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent, action: EditorAction) {
        match key.code {
            KeyCode::Esc => {
                self.cancel_editor();
            }

            KeyCode::Enter => {
                self.submit_editor(action);
            }

            KeyCode::Backspace => {
                self.input.pop();
            }

            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.input.push(character);
            }

            _ => {}
        }
    }

    fn handle_delete_confirmation(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.delete_selected();
            }

            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.set_status(StatusKind::Info, "Deletion cancelled");
            }

            _ => {}
        }
    }

    fn start_add(&mut self) {
        self.input.clear();
        self.mode = Mode::Editing(EditorAction::Add);
    }

    fn start_rename(&mut self) {
        let Some(index) = self.selected_index() else {
            self.set_status(StatusKind::Error, "No task is selected");
            return;
        };

        self.input = self.todos[index].title.clone();
        self.mode = Mode::Editing(EditorAction::Rename);
    }

    fn start_delete(&mut self) {
        if self.selected_index().is_none() {
            self.set_status(StatusKind::Error, "No task is selected");
            return;
        }

        self.mode = Mode::ConfirmDelete;
    }

    fn cancel_editor(&mut self) {
        self.input.clear();
        self.mode = Mode::Normal;

        self.set_status(StatusKind::Info, "Editing cancelled");
    }

    fn submit_editor(&mut self, action: EditorAction) {
        let title = self.input.trim().to_string();

        if title.is_empty() {
            self.set_status(StatusKind::Error, "The title cannot be empty");
            return;
        }

        match action {
            EditorAction::Add => self.add_todo(title),
            EditorAction::Rename => self.rename_selected(title),
        }
    }

    fn add_todo(&mut self, title: String) {
        let previous = self.todos.clone();

        let next_id = self.todos.iter().map(|todo| todo.id).max().unwrap_or(0) + 1;

        self.todos.push(Todo::new(next_id, title.clone()));

        if self.persist(previous, format!("Added: {title}")) {
            self.table_state.select(Some(self.todos.len() - 1));

            self.close_editor();
        }
    }

    fn rename_selected(&mut self, title: String) {
        let Some(index) = self.selected_index() else {
            self.set_status(StatusKind::Error, "No task is selected");
            return;
        };

        let previous = self.todos.clone();

        self.todos[index].title = title.clone();

        if self.persist(previous, format!("Renamed: {title}")) {
            self.close_editor();
        }
    }

    fn toggle_selected(&mut self) {
        let Some(index) = self.selected_index() else {
            self.set_status(StatusKind::Error, "No task is selected");
            return;
        };

        let previous = self.todos.clone();

        self.todos[index].toggle();

        let title = self.todos[index].title.clone();
        let finished = self.todos[index].finished;

        let message = if finished {
            format!("Completed: {title}")
        } else {
            format!("Reopened: {title}")
        };

        self.persist(previous, message);
    }

    fn delete_selected(&mut self) {
        let Some(index) = self.selected_index() else {
            self.mode = Mode::Normal;
            return;
        };

        let previous = self.todos.clone();
        let deleted = self.todos.remove(index);

        self.mode = Mode::Normal;

        if self.persist(previous, format!("Deleted: {}", deleted.title)) {
            self.normalize_selection();
        }
    }

    fn persist(&mut self, previous: Vec<Todo>, success_message: String) -> bool {
        match storage::save_todos(&self.path, &self.todos) {
            Ok(()) => {
                self.set_status(StatusKind::Success, success_message);

                true
            }

            Err(error) => {
                self.todos = previous;
                self.normalize_selection();

                self.set_status(
                    StatusKind::Error,
                    format!("Could not save changes: {error}"),
                );

                false
            }
        }
    }

    fn close_editor(&mut self) {
        self.input.clear();
        self.mode = Mode::Normal;
    }

    fn set_status(&mut self, kind: StatusKind, message: impl Into<String>) {
        self.status = Status {
            kind,
            message: message.into(),
        };
    }

    fn selected_index(&self) -> Option<usize> {
        self.table_state
            .selected()
            .filter(|index| *index < self.todos.len())
    }

    fn select_next(&mut self) {
        if self.todos.is_empty() {
            self.table_state.select(None);
            return;
        }

        let next = match self.table_state.selected() {
            Some(index) if index + 1 < self.todos.len() => index + 1,

            _ => 0,
        };

        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.todos.is_empty() {
            self.table_state.select(None);
            return;
        }

        let previous = match self.table_state.selected() {
            Some(0) | None => self.todos.len() - 1,
            Some(index) => index - 1,
        };

        self.table_state.select(Some(previous));
    }

    fn select_first(&mut self) {
        if self.todos.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if self.todos.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(self.todos.len() - 1));
        }
    }

    fn normalize_selection(&mut self) {
        if self.todos.is_empty() {
            self.table_state.select(None);
            return;
        }

        let selected = self
            .table_state
            .selected()
            .unwrap_or_default()
            .min(self.todos.len() - 1);

        self.table_state.select(Some(selected));
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn todos(&self) -> &[Todo] {
        &self.todos
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn selected_todo(&self) -> Option<&Todo> {
        self.selected_index()
            .and_then(|index| self.todos.get(index))
    }

    pub fn completed_count(&self) -> usize {
        self.todos.iter().filter(|todo| todo.finished).count()
    }

    pub fn pending_count(&self) -> usize {
        self.todos.len().saturating_sub(self.completed_count())
    }

    pub(crate) fn table_parts(&mut self) -> (&[Todo], &mut TableState) {
        (&self.todos, &mut self.table_state)
    }
}
