mod core;
mod web;

use anyhow::Result;
use std::path::PathBuf;

 #[tokio::main]
 async fn main() -> Result<()> {
     let db_path = PathBuf::from("osu_maps.db");
     web::serve(&db_path).await?;
     Ok(())
 }