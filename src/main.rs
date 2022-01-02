use actix_web::{App, get, HttpResponse, HttpServer, post, ResponseError, web};
use actix_web::http::header;
use askama::Template;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render")]
    AskamaError(#[from] askama::Error),

    #[error("Failed to get connection")]
    ConncectionPoolError(#[from] r2d2::Error),

    #[error("Failed SQL execution")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for MyError {}

struct TodoEntry {
    id: i32,
    text: String
}

#[derive(Template)]
#[template(path="index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>
}

#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    let mut statement = conn.prepare("SELECT id, text from todo")?;

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
    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[derive(Deserialize)]
struct AppParams {
    text: String,
}

#[post("/add")]
async fn add(
    param: web::Form<AppParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    conn.execute("INSERT into TODO (text) values (?)", &[&param.text])?;
    Ok(HttpResponse::SeeOther()
        .header(header::LOCATION, "/")
        .finish())
}

#[derive(Deserialize)]
struct DeleteParams {
    id: i32,
}

#[post("/delete")]
async fn delete(
    param: web::Form<DeleteParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    conn.execute("delete from TODO where id = (?)", &[&param.id])?;
    Ok(HttpResponse::SeeOther()
        .header(header::LOCATION, "/")
        .finish())
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let manager = SqliteConnectionManager::file("todo.db");
    let pool = Pool::new(manager).expect("Failed to initialize");
    let connection = pool.get().expect("failed to get connection from pool");
    connection.execute("CREATE table if not EXISTS todo (
        id integer primary key autoincrement,
        text text not null
    )",
                       params![],
    ).expect("failed initial sql");

    HttpServer::new(move ||
        App::new()
            .service(index)
            .service(add)
            .service(delete)
            .data(pool.clone())
    )
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}



