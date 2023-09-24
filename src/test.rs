use actix_web::{body::to_bytes, dev::Service, http, test, web, App, Error};

use super::*;

#[actix_web::test]
async fn test_index() -> Result<(), Error> {
    let app = App::new().route("/", web::get().to(index));
    let app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = app.call(req).await?;

    assert_eq!(resp.status(), http::StatusCode::OK);

    let response_body = resp.into_body();
    assert_eq!(to_bytes(response_body).await?, r##"Hello world!"##);

    Ok(())
}

#[actix_web::test]
async fn test_health_check_handler() -> Result<(), Error> {
    let app = App::new().route("/health", web::get().to(health_check_handler));
    let app = test::init_service(app).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = app.call(req).await?;

    assert_eq!(resp.status(), http::StatusCode::OK);

    Ok(())
}
