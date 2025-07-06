use tokio_postgres::{Client, Error};
use tracing::info;

pub async fn upsert_lottery_operator(client: &Client, name: &str) -> Result<(), Error> {
    let upsert_operator_query = "
        INSERT INTO lottery_operators (name) VALUES ($1)
        ON CONFLICT (name) DO UPDATE SET name = $1
    ";
    client.execute(upsert_operator_query, &[&name]).await?;
    info!("Upserted lottery operator: {}", name);
    Ok(())
}
