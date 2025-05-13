use actix_web::{App, HttpServer, Responder, web};

async fn index() -> impl Responder {
    "Welcome to the Rootstock Wallet UI!"
}

pub async fn start_web_server() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";

    println!("Starting Actix Web server at http://{}", addr);

    HttpServer::new(|| {
        App::new().route("/", web::get().to(index)) // Define the index route
    })
    .bind(addr)?
    .run()
    .await
}
