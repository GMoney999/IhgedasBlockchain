// use actix_web::{get, web, post, Responder, HttpResponse};
// use lazy_static::{lazy_static};
// use tera::{Tera};
// use crate::wallet::{Wallets};
// use actix_session::{Session};
// use crate::models::blockchain::{Blockchain};
// use crate::utxoset::{UTXOSet};
// use bitcoincash_addr::{Address};
//
// lazy_static! {
//     pub static ref TEMPLATES: Tera = {
//         let source = "templates/**/*";
//         let mut tera = Tera::new(source).unwrap();
//         tera
//     };
// }
//
// #[get("/")]
// pub async fn health_check() -> String {
//     "This is a health check".to_string()
// }
//
// // Blockchain home page before wallet is created
// #[get("/ihgedas")]
// pub async fn index() -> impl Responder {
//     let context = tera::Context::new();
//     let page_content = TEMPLATES.render("layout.html", &context).unwrap();
//     HttpResponse::Ok().body(page_content)
// }
//
// // Create a wallet, add it to database, return new HTML
// #[post("/create-wallet")]
// pub async fn create_wallet(tera: web::Data<Tera>, session: Session) -> impl Responder {
//     // Create a group of wallets to keep track of every wallet
//     let mut ws = match Wallets::new() {
//         Ok(ws) => ws,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Error initializing wallets: {}", e))
//     };
//
//     // Create a new wallet and get its address
//     let address = ws.create_wallet();
//
//     let funds: i32 = 100;
//
//     // Save the wallet address and funds to session context
//     session.insert("wallet_address", &address).unwrap();
//     session.insert("wallet_funds", &funds).unwrap();
//
//
//     if let Err(e) = ws.save_all() {
//         return HttpResponse::InternalServerError().body(format!("Error saving wallet data: {}", e))
//     };
//
//     let tera_context = tera::Context::new();
//
//
//     let display_content = match tera.render("display.html", &tera_context) {
//         Ok(content) => content,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Template error: {}", e))
//     };
//     let controller_content = match tera.render("controller.html", &tera_context) {
//         Ok(content) => content,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Template error: {}", e))
//     };
//
//     let new_main_content = format!(
//         r#"<main>
//             <header class='header'>
//                 <h1>Ihgedas Blockchain</h1>
//                 <p id='wallet-address'>Current Wallet Address: <strong>{}</strong></p>
//                 <p id='wallet-funds'>Wallet Funds: <strong>{}</strong></p>
//             </header>
//             <div class='display'>
//                 {}
//             </div>
//             <div class='controller'>
//                 {}
//             </div>
//         </main>"#,
//         address, funds, display_content, controller_content
//     );
//
//     HttpResponse::Ok().body(new_main_content)
// }
//
// #[post("/create-blockchain")]
// pub async fn create_blockchain(tera: web::Data<Tera>, session: Session) -> impl Responder {
//     let wallet_address = match session.get::<String>("wallet_address") {
//         Ok(Some(address)) => address,
//         Err(_) | Ok(None) => return HttpResponse::BadRequest().body("No wallet address found in session"),
//     };
//
//     let bc = match Blockchain::create_blockchain(wallet_address) {
//         Ok(bc) => bc,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create blockchain: {:?}", e)),
//     };
//
//     let utxo_set = UTXOSet { blockchain: bc.clone() };
//
//     if let Err(e) = utxo_set.reindex() {
//         return HttpResponse::InternalServerError().body(format!("Failed to reindex UTXOs: {:?}", e));
//     }
//
//     let blocks = match bc.get_blocks() {
//         Ok(blocks) => blocks,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to get blocks: {}", e)),
//     };
//
//     let mut context = tera::Context::new();
//     context.insert("blocks", &blocks);
//
//     let content = match tera.render("block.html", &context) {
//         Ok(content) => content,
//         Err(e) => return HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
//     };
//
//     HttpResponse::Ok().body(content)
// }
//
//
//
//
//
//
// pub async fn get_balance_for_address(address: &str) -> Result<i32, Box<dyn std::error::Error>> {
//     let pub_key_hash =  Address::decode(address).unwrap().body;
//     let bc = Blockchain::new().unwrap();
//     let utxo_set = UTXOSet { blockchain: bc };
//     let utxos = utxo_set.find_utxos(&pub_key_hash).unwrap();
//     let balance: i32 = utxos.outputs.iter().map(|out| out.value).sum();
//     Ok(balance)
// }
//
//
