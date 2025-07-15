use tokio_postgres::{Client, Error};
use uuid::Uuid;
use tracing::{info};
use crate::core::wager::{Wager};
use crate::core::board::Board;
use crate::core::selection::Selection;

pub async fn insert_wager(client: &Client, wager: &Wager) -> Result<(), Error> {
    info!("Attempting to insert wager: {:?}", wager);
    client
        .execute(
            "INSERT INTO wagers (id, user_id, draw_id, game_type, system_game_level, stake, price, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[&wager.id, &wager.user_id, &wager.draw_id, &wager.game_type.to_string(), &wager.system_game_level, &wager.stake, &wager.price, &wager.created_at],
        )
        .await?;
    info!("Successfully inserted wager: {:?}", wager);
    Ok(())
}

pub async fn insert_board(client: &Client, board: &Board) -> Result<(), Error> {
    info!("Attempting to insert board: {:?}", board);
    client
        .execute(
            "INSERT INTO boards (id, wager_id) VALUES ($1, $2)",
            &[&board.id, &board.wager_id],
        )
        .await?;
    info!("Successfully inserted board: {:?}", board);
    Ok(())
}

pub async fn insert_selection(client: &Client, selection: &Selection, board_id: Uuid) -> Result<(), Error> {
    info!("Attempting to insert selection: {:?}", selection);
    client
        .execute(
            "INSERT INTO selections (id, board_id, draw_level_id, value) VALUES ($1, $2, $3, $4)",
            &[&selection.id, &board_id, &selection.draw_level_id, &selection.value],
        )
        .await?;
    info!("Successfully inserted selection: {:?}", selection);
    Ok(())
}
