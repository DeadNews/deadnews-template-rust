use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Log the start of the HTTP server
    log::info!("Starting HTTP server at http://127.0.0.1:1271.");

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
    .bind(("127.0.0.1", 1271))?
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
