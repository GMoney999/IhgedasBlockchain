use crate::cli::Cli;
use crate::error::{Result};
// use tera::{Tera};
// use crate::server::{index, health_check, create_wallet, create_blockchain};
// use actix_files as fs;
// use actix_web::middleware::{Logger};
// use actix_session::{SessionMiddleware};
// use actix_session::storage::{CookieSessionStore};
// use actix_web::{web, App, HttpServer};
// use actix_web::cookie::{Key};
// use dotenv::dotenv;

mod models;
mod server;
mod tx;
mod utils;
mod wallet;
mod error;
mod transaction;
mod utxoset;
mod cli;
mod contracts;

fn main() -> Result<()>{
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}


// fn get_secret_key() -> Key {
//     dotenv().ok(); // Load environment variables from .env file
//
//     let key = std::env::var("SESSION_KEY").expect("SESSION_KEY must be set and 128 characters long (64 bytes)");
//     assert_eq!(key.len(), 128, "Session key must be exactly 128 characters long");
//
//     // Convert hex string to bytes
//     let key_bytes = hex::decode(key).expect("Session key must be a valid hex string");
//     Key::from(&key_bytes)
// }
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     std::env::set_var("RUST_LOG", "debug");
//     std::env::set_var("RUST_BACKTRACE", "1");
//     env_logger::init();
//
//     // let secret_key = get_secret_key();
//
//     let tera = web::Data::new(Tera::new("templates/**/*").unwrap());
//
//     HttpServer::new(move || {
//         let logger = Logger::default();
//
//         App::new()
//             .app_data(tera.clone())
//             .wrap(logger)
//             // .wrap(SessionMiddleware::new(
//             //     CookieSessionStore::default(),
//             //     secret_key.clone()
//             // ))
//             .service(health_check)
//             .service(index)
//             .service(create_wallet)
//             .service(create_blockchain)
//             .service(fs::Files::new("/assets", "./assets"))
//             .service(fs::Files::new("/style", "./style"))
//     })
//         .bind(("127.0.0.1", 7878))?
//         .run()
//         .await
// }