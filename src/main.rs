#[macro_use]
extern crate diesel;

use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

// We define a custom type for connection pool to use later.
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

mod handlers;
mod models;
mod schema;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    // Loading .env into environment variable.
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(|| async { "Actix REST API" }))
            .service(handlers::index)
            .service(handlers::create)
            .service(handlers::show)
            .service(handlers::update)
            .service(handlers::destroy)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
