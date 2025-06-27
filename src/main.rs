use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Get the port from the environment variable or use the default port 8000
    let port: u16 = env::var("SERVICE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000);

    // Log the start of the HTTP server
    log::info!("Starting HTTP server at http://0.0.0.0:{port}.");

    // Create and run the HTTP server
    HttpServer::new(|| {
        App::new()
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            // Define routes
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
            .service(web::resource("/health").to(health_check_handler))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn index(req: HttpRequest) -> &'static str {
    println!("REQ: {req:?}");
    "Hello world!"
}

async fn health_check_handler() -> HttpResponse {
    HttpResponse::Ok().json("Healthy!")
}

#[cfg(test)]
mod test;
