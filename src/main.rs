use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct Todo {
    id: String,
    task: String,
    completed: bool,
}

#[derive(Serialize, Deserialize)]
struct NewTodo {
    task: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateTodo {
    task: String,
    completed: bool,
}

#[derive(Serialize, Deserialize)]
struct UpdateTodoRequest {
    todo: UpdateTodo
}

#[derive(Serialize, Deserialize)]
struct TodoListResponse {
    todos: Vec<Todo>,
}

#[derive(Serialize, Deserialize)]
struct TodoResponse {
    todo: Todo,
}

#[derive(Serialize, Deserialize)]
struct Response {
    message: String,
}

struct AppState {
    todos: Mutex<Vec<Todo>>,
}

async fn get_todos(data: web::Data<AppState>) -> impl Responder {
    let todos = data.todos.lock().unwrap();
    let response = TodoListResponse {
        todos: todos.clone(),
    };
    HttpResponse::Ok().json(response)
}

async fn get_todo_by_id(data: web::Data<AppState>, todo_id: web::Path<String>) -> impl Responder {
    let todos = data.todos.lock().unwrap();
    let todo_id = todo_id.into_inner();
    match todos.iter().find(|&todo| todo.id == todo_id) {
        Some(todo) => {
            let todo = TodoResponse {
                todo: todo.clone()
            };
            HttpResponse::Ok().json(todo)
        },
        None => HttpResponse::NotFound().body("Todo not found"),
    }
}

async fn create_todo(data: web::Data<AppState>, new_todo: web::Json<NewTodo>) -> impl Responder {
    let mut todos = data.todos.lock().unwrap();
    let response = TodoResponse {
        todo: Todo {
            id: Uuid::new_v4().to_string(),
            task: new_todo.task.clone(),
            completed: false,
        }
    };
    todos.push(response.todo.clone());
    HttpResponse::Created().json(response)
}

async fn update_todo(
    data: web::Data<AppState>,
    todo_id: web::Path<String>,
    updated_todo: web::Json<UpdateTodoRequest>,
) -> impl Responder {
    let mut todos = data.todos.lock().unwrap();
    let todo_id = todo_id.into_inner();
    match todos.iter_mut().find(|todo| todo.id == todo_id) {
        Some(todo) => {
            todo.task = updated_todo.todo.task.clone();
            todo.completed = updated_todo.todo.completed;
            let todo = TodoResponse {
                todo: todo.clone()
            };
            HttpResponse::Ok().json(todo)
        }
        None => HttpResponse::NotFound().body("Todo not found"),
    }
}

async fn delete_todo(
    data: web::Data<AppState>,
    todo_id: web::Path<String>,
) -> impl Responder {
    let mut todos = data.todos.lock().unwrap();
    let todo_id = todo_id.into_inner();
    if let Some(pos) = todos.iter().position(|todo| todo.id == todo_id) {
        todos.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().body("Todo not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        todos: Mutex::new(vec![]),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::NormalizePath::trim())
            .route("/todos", web::get().to(get_todos))
            .route("/todos/{id}", web::get().to(get_todo_by_id))
            .route("/todos", web::post().to(create_todo))
            .route("/todos/{id}", web::put().to(update_todo))
            .route("/todos/{id}", web::delete().to(delete_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
