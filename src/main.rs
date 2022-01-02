use actix_web::{get, post, App, HttpResponse, HttpServer, ResponseError};

use askama::Template;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render")]
    AskamaError(#[from] askama::Error),
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
async fn index() -> Result<HttpResponse, MyError>{
    let mut entries = Vec::new();

    entries.push(TodoEntry{
        id: 1,
        text: String::from("katsumaru")
    });
    entries.push(TodoEntry{
        id: 2,
        text: String::from("Minami")
    });
    let html = IndexTemplate { entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[post("/add")]
async fn add() -> Result<HttpResponse, MyError>{
    unimplemented!()
}

#[post("/delete")]
async fn delete() -> Result<HttpResponse, MyError>{
    unimplemented!()
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error>{
    HttpServer::new(move || App::new().service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}



