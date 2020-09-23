use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, ResponseError};
use thiserror::Error;
use askama::Template;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use rusqlite::params;

struct TodoEntry {
    id: u32,
    text: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),

    #[error("Failed to get connextion")]
    ConnectionPoolError(#[from] r2d2::Error),

    #[error("Failed SQL execution")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for MyError {}

#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    let connect = db.get()?;

    let mut statement = connect.prepare("SELECT id, text FROM todo")?;

    let rows = statement.query_map(params![], |row| {
        let id = row.get(0)?;
        let text = row.get(1)?;
        Ok(TodoEntry { id, text })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    let html = IndexTemplate { entries };
    let response_body = html.render()?;

    Ok(HttpResponse::Ok().content_type("text.html").body(response_body))
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let manager = SqliteConnectionManager::file("todo.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let connection = pool.get().expect("Failed to get the connection from the pool");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS todo(id INTEGER PRIMARY KEY AUTOINCREMENT,text TEXT NOT NULL)",
        params![],
    ).expect("Failed to create a table 'todo'.");

    HttpServer::new(move|| App::new().service(index).data(pool.clone()))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}