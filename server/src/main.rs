use std::sync::Mutex;

use actix_files::Files;
use actix_web::{
    get, middleware::Logger, web, web::Data, App, HttpResponse, HttpServer, Responder,
};
use env_logger::Env;
use log::info;
use my_web_app::MyTestStruct;

#[get("/hello")]
async fn hello() -> impl Responder {
    info!("Sending a String.");
    "Hello Cindy"
}

#[get("/json-data")]
async fn jsondata(counter: Data<Mutex<i32>>) -> impl Responder {
    let mut v = counter.lock().unwrap();
    *v += 1;
    let data = MyTestStruct::from(*v);
    info!("Data: {:?}", data);
    info!("Sending: {:?}", counter);
    serde_json::to_string(&data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        Env::default()
            .default_filter_or("info")
            .default_filter_or("debug"),
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a i"))
            .app_data(Data::new(Mutex::new(0)))
            .service(hello)
            .service(jsondata)
            .service(Files::new("/", "./../dist/").index_file("index.html"))
            .default_service(web::to(|| async {
                HttpResponse::Found()
                    .append_header(("Location", "/"))
                    .finish()
            }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
