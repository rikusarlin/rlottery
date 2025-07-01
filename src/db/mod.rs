use refinery::{Error, Report};

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub async fn run_migrations(client: &mut tokio_postgres::Client) -> Result<Report, Error> {
    migrations::migrations::runner().run_async(client).await
}
