use actix_web::{web::{self, Json, Path}, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use sqlx::{prelude::FromRow, Executor,SqlitePool};

#[actix_web::main]
async fn main() {
    let pool = db().await;

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(pool.clone()))
        .route("/todo_list", web::get().to(get_todo_list))
        .route("/add_todo", web::post().to(add_todo))
        .route("/todo/{id}", web::get().to(get_single_todo))
        .route("/delete_todo/{id}", web::delete().to(delete_todo))
        .route("/update_todo/{id}", web::put().to(update_todo))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await
    .unwrap()
}


async fn db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite://db.sqlite").await.unwrap();

    pool.execute("
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT
        )
    ").await.unwrap();
    pool
}

async fn get_todo_list(pool: web::Data<SqlitePool>) -> impl Responder {
    let todos: Vec<Todo> = sqlx::query_as("SELECT * FROM todos").fetch_all(pool.get_ref()).await.unwrap();

    let todos_json = serde_json::to_string(&todos).unwrap();

    HttpResponse::Ok().body(todos_json)
}

async fn add_todo(todo: Json<TodoRequest>, pool: web::Data<SqlitePool>) -> impl Responder {
    sqlx::query("INSERT INTO todos (content) VALUES (?1)").bind(&todo.content).execute(pool.get_ref()).await.unwrap();

    HttpResponse::Ok().body("Added new Todo")
}

async fn get_single_todo(id:Path<i32>, pool: web::Data<SqlitePool>) -> impl Responder {
    let id = id.into_inner();
    let rows: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1").bind(&id).fetch_all(pool.get_ref()).await.unwrap();

    if rows.len() == 0 {
        let msg = format!("NO todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        let todo_json = serde_json::to_string(&rows[0]).unwrap();
        HttpResponse::Ok().body(todo_json)
    }
}

async fn delete_todo(id: Path<i32>, pool: web::Data<SqlitePool>) -> impl Responder {
    let id = id.into_inner();
    let rows: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1").bind(&id).fetch_all(pool.get_ref()).await.unwrap();

    if rows.len() == 0 {
        let msg = format!("No todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        sqlx::query("DELETE FROM todos WHERE id= ?1").bind(&id).execute(pool.get_ref()).await.unwrap();
        HttpResponse::Ok().body("Deleted todo successfully")
    }
}

async fn update_todo(id:Path<i32>, pool: web::Data<SqlitePool>, todo:Json<TodoRequest>) -> impl Responder {
    let id = id.into_inner();
    let rows: Vec<Todo> = sqlx::query_as("SELECT * FROM todos WHERE id = ?1").bind(&id).fetch_all(pool.get_ref()).await.unwrap();

    if rows.len() == 0 {
        let msg = format!("NO todo id: {} found!", id);
        HttpResponse::NotFound().body(msg)
    } else {
        sqlx::query("UPDATE todos SET content = ?1 WHERE id = ?2").bind(&todo.content).bind(&id).execute(pool.get_ref()).await.unwrap();
        HttpResponse::Ok().body("updated to successfully")
    }
}


#[derive(Serialize, FromRow)]
struct Todo {
    id: i32,
    content: String
}

#[derive(Deserialize)]
struct TodoRequest {
    content: String
}