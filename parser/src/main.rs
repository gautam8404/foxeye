use anyhow::Result;

use crate::parser::Parser;

mod config;
mod parser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    println!("Hello, world!");

    let parser = Parser::new().await.unwrap();

    parser.parser_loop().await;

    Ok(())
}
