use std::io::Write;
use actix_web::{get, post, web, App, HttpResponse, Error, HttpServer, Result};
use actix_multipart::{Field, Multipart};
use tera::{Tera, Context};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::str;

mod risc16;

#[derive(Deserialize, Serialize, Debug)]
struct Exe {
    exo: String,
    archi: String,
    logic: i32,
    asm_digest: String,
    exec: i32,
}

#[get("/")]
async fn hello(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context.insert("product", &"igloo");
    context.insert("option", &"COUcou");
    let s = tmpl.render("index.html", &context);
    Ok(HttpResponse::Ok().content_type("text/html").body(s.unwrap()))
}

#[post("/")]
async fn post(tmpl: web::Data<tera::Tera>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    let mut filetext = String::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();
        // let mut data = Bytes::new();
        if name == "file" {
            match content_type.get_filename() {
                Some(filename) => {
                    // let tmp_file = Tmpfile::new(&sanitize_filename::sanitize(&filename));
                    // let tmp_path = tmp_file.tmp_path.clone();
                    // let mut f = web::block(move || std::fs::File::create(&tmp_path))
                    //     .await
                    //     .unwrap();
                    while let Some(chunk) = field.next().await {
                        filetext.push_str(str::from_utf8(&chunk?)?);
                    }
                    // tmp_files.push(tmp_file.clone());
                    //
                println!("{} file", filename);
                }
                None => {
                    println!("file none");
                }
            }
        } else {
            while let Some(chunk) = field.next().await {
                let data = chunk.expect(" split_payload err chunk");
                context.insert(name, str::from_utf8(&data).unwrap());
            }
        } 
    }


    let log = risc16::main_from_str(&filetext);

    context.insert("log_content", &log);
    context.insert("code_content", &filetext);
    context.insert("status", &"done");
    println!("{:?}", context);
    let s = tmpl.render("log_done.html", &context);
    Ok(HttpResponse::Ok().content_type("text/html").body(s.unwrap()))
    // Ok(HttpResponse::Ok().body(format!("Welcome {}!","test")))
}


async fn manual_hello(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    context.insert("product", &"igloo");
    context.insert("option", &"COUcou");
    let s = tmpl.render("index.html", &context);
    Ok(HttpResponse::Ok().content_type("text/html").body(s.unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        let tera =
            Tera::new("templates/**/*.html").unwrap();
        App::new()
            .data(tera)
            .service(hello)
            .service(post)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
