use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};

use crate::app::{App, Mode, Task, View};

pub fn open_and_init(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tasks (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            sort_order      INTEGER NOT NULL DEFAULT 0,
            name            TEXT NOT NULL,
            notes           TEXT,
            project         TEXT,
            completed       INTEGER NOT NULL DEFAULT 0,
            pomodoros       INTEGER NOT NULL DEFAULT 0,
            time_spent_secs INTEGER NOT NULL DEFAULT 0,
            creation_date   TEXT NOT NULL,
            completion_date TEXT
        );
        CREATE TABLE IF NOT EXISTS app_state (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
}

fn get_state(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_state WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

pub struct LoadedState {
    pub tasks: Vec<Task>,
    pub mode: Mode,
    pub pomodoros_total: u32,
    pub current_view: View,
    pub active_task_index: Option<usize>,
    pub time_remaining_secs: Option<u64>,
}

pub fn load_from(conn: &Connection) -> LoadedState {
    let tasks = load_tasks(conn).unwrap_or_default();
    let mode = get_state(conn, "mode")
        .and_then(|s| match s.as_str() {
            "ShortBreak" => Some(Mode::ShortBreak),
            "LongBreak" => Some(Mode::LongBreak),
            _ => Some(Mode::Pomodoro),
        })
        .unwrap_or_default();
    let pomodoros_total: u32 = get_state(conn, "pomodoros_total")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let current_view = get_state(conn, "current_view")
        .and_then(|s| match s.as_str() {
            "Timer" => Some(View::Timer),
            "Statistics" => Some(View::Statistics),
            _ => Some(View::TaskList),
        })
        .unwrap_or_default();
    let active_task_index = get_state(conn, "active_task_index")
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&i| i < tasks.len());
    let time_remaining_secs = get_state(conn, "time_remaining_secs")
        .and_then(|s| s.parse::<u64>().ok());
    LoadedState { tasks, mode, pomodoros_total, current_view, active_task_index, time_remaining_secs }
}

fn load_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT name, notes, project, completed, pomodoros, time_spent_secs, creation_date, completion_date
         FROM tasks ORDER BY sort_order ASC",
    )?;
    let tasks = stmt
        .query_map([], |row| {
            let creation_str: String = row.get(6)?;
            let completion_str: Option<String> = row.get(7)?;
            Ok(Task {
                name: row.get(0)?,
                notes: row.get(1)?,
                project: row.get(2)?,
                completed: row.get::<_, i64>(3)? != 0,
                pomodoros: row.get::<_, i64>(4)? as u32,
                time_spent: Duration::from_secs(row.get::<_, i64>(5)? as u64),
                creation_date: creation_str
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                completion_date: completion_str.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tasks)
}

pub fn save_to(conn: &mut Connection, app: &App) -> Result<()> {
    let tx = conn.transaction()?;
    save_tasks(&tx, &app.tasks)?;
    save_app_state(&tx, app)?;
    tx.commit()
}

fn save_tasks(conn: &Connection, tasks: &[Task]) -> Result<()> {
    conn.execute("DELETE FROM tasks", [])?;
    for (i, task) in tasks.iter().enumerate() {
        conn.execute(
            "INSERT INTO tasks (sort_order, name, notes, project, completed, pomodoros, time_spent_secs, creation_date, completion_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                i as i64,
                task.name,
                task.notes,
                task.project,
                task.completed as i64,
                task.pomodoros as i64,
                task.time_spent.as_secs() as i64,
                task.creation_date.to_rfc3339(),
                task.completion_date.map(|d| d.to_rfc3339()),
            ],
        )?;
    }
    Ok(())
}

fn save_app_state(conn: &Connection, app: &App) -> Result<()> {
    let mode_str = match app.mode {
        Mode::Pomodoro => "Pomodoro",
        Mode::ShortBreak => "ShortBreak",
        Mode::LongBreak => "LongBreak",
    };
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('mode', ?1)",
        params![mode_str],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('pomodoros_total', ?1)",
        params![app.pomodoros_completed_total as i64],
    )?;
    let view_str = match app.current_view {
        View::Timer => "Timer",
        View::TaskList => "TaskList",
        View::Statistics => "Statistics",
        View::Settings => "Settings",
        View::TaskDetails => "TaskDetails",
    };
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('current_view', ?1)",
        params![view_str],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('time_remaining_secs', ?1)",
        params![app.time_remaining.as_secs() as i64],
    )?;
    match app.active_task_index {
        Some(idx) => {
            conn.execute(
                "INSERT OR REPLACE INTO app_state (key, value) VALUES ('active_task_index', ?1)",
                params![idx as i64],
            )?;
        }
        None => {
            conn.execute("DELETE FROM app_state WHERE key = 'active_task_index'", [])?;
        }
    }
    Ok(())
}
