use std::path::Path;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use listenfd::ListenFd;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use dotenv::dotenv;
use tokio::fs as tfs;
mod routes;

pub struct AppState {
    pub db: Pool<Postgres>,
    }
    

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !Path::new("./upload").exists() {
        tfs::create_dir("./upload").await?;
    }
    dotenv().ok();
    let database_url =
        std::env::var("POSTGRES_DB_PROPERTIES").expect("POSTGRES_DB_PROPERTIES must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1000)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(routes::upload)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host =
                std::env::var("HOST_STAFF_PROPERTIES").expect("HOST_STAFF_PROPERTIES must be set");
            let port =
                std::env::var("PORT_STAFF_PROPERTIES").expect("PORT_STAFF_PROPERTIES must be set");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await
}
