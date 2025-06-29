use refinery::{Error, Report};

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub fn run_migrations(client: &mut postgres::Client) -> Result<Report, Error> {
    migrations::migrations::runner().run(client)
}
