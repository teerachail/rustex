use std::collections::HashMap;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // your code here
    println!("Hello, world!");

    // make an HTTP call using reqwest
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{resp:#?}");

    Ok(())
}