use llp_indexer::handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler().await.unwrap(); // TODO: Handle error required

    Ok(())
}
