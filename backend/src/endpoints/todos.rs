use std::fs::read_to_string;

use crate::{ApiResponse, AppState};
use axum::{
    extract::Path,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Router,
};
use serde::{Deserialize, Serialize};

const URL: &str = "/todos";
const URL_ID: &str = "/todos/:id";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToDo {
    id: i64,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewToDo {
    title: String,
    description: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditToDo {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    count: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        // `GET /` goes to `root`
        .route(URL, routing::get(get_all_todos))
        .route(URL, routing::post(post))
        .route(URL_ID, routing::get(get))
        .route(URL_ID, routing::put(put))
        .route(URL_ID, routing::patch(patch))
        .route(URL_ID, routing::delete(delete))
}

pub fn read_file() -> anyhow::Result<Vec<ToDo>> {
    let file_content = read_to_string("todos.json")?;
    let list = serde_json::from_str::<Vec<ToDo>>(&file_content)?;
    Ok(list)
}

pub fn write(list: Vec<ToDo>) -> anyhow::Result<()> {
    let file_content = serde_json::to_string(&list)?;
    std::fs::write("todos.json", file_content)?;
    Ok(())
}

async fn get_all_todos(State(state): State<AppState>) -> ApiResponse {
    let result = sqlx::query_as!(ToDo, "SELECT * FROM todos")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(result).into_response())
}

async fn get(Path(id): Path<i64>) -> ApiResponse {
    let array = read_file().unwrap();
    let todo = array.iter().find(|x| x.id == id);
    match todo {
        Some(x) => Ok(Json(x).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn post(Json(payload): Json<NewToDo>) -> ApiResponse {
    let mut array: Vec<ToDo> = read_file().unwrap();
    let id = array.iter().map(|x| x.id).max().unwrap_or(0) + 1;
    let todo = ToDo {
        id: id,
        title: payload.title,
        description: payload.description,
        completed: payload.completed,
    };
    array.push(todo.clone());
    write(array).unwrap();
    Ok(Json(todo).into_response())
}

async fn patch(Path(id): Path<i64>, Json(payload): Json<EditToDo>) -> ApiResponse {
    let mut array = read_file().unwrap(); // Leggi array dei todos
    let index = array.iter().position(|x| x.id == id);
    if index.is_none() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let todo = &mut array[index.unwrap()];
    let todo = ToDo {
        id: todo.id,
        title: payload.title.unwrap_or(todo.title.clone()),
        description: payload.description.unwrap_or(todo.description.clone()),
        completed: payload.completed.unwrap_or(todo.completed.clone()),
    };
    array[index.unwrap()] = todo.clone();
    write(array).expect("Gianni");
    Ok(Json(todo).into_response())
}

async fn put(Path(id): Path<i64>, Json(payload): Json<NewToDo>) -> ApiResponse {
    let mut array: Vec<ToDo> = read_file().unwrap();
    let index = array.iter().position(|x| x.id == id);
    if index.is_none() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let todo = ToDo {
        id: id,
        title: payload.title,
        description: payload.description,
        completed: payload.completed,
    };
    array[index.unwrap()] = todo.clone();
    write(array).expect("Gianni");
    Ok(Json(todo).into_response())
}

async fn delete(Path(id): Path<i64>) -> ApiResponse {
    let mut array = read_file().unwrap(); // Leggi array dei todos
    let index = array.iter().position(|x| x.id == id);
    if index.is_none() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    array.remove(index.unwrap());
    write(array).expect("Gianni"); // Riscrivi array
    Ok(StatusCode::NO_CONTENT.into_response())
}
