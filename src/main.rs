use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let a = reqwest::get("https://httpbin.org/ip")
        .await
        .unwrap()
        .json::<HashMap<String, String>>()
        .await
        .unwrap();
    println!("{a:#?}");
}
