use tokio_postgres::{Client, Error};
use uuid::Uuid;
use tracing::{info};
use crate::core::wager::{Wager};
use crate::core::board::Board;
use crate::core::selection::Selection;

pub async fn insert_wager(client: &Client, wager: &Wager, draws: Vec<i32>) -> Result<(), Error> {
    info!("Attempting to insert wager: {:?} to draws {:?}", wager, draws);
    let stake = wager.stake as i32;
    let price = wager.price as i32;
    client
        .execute(
            "INSERT INTO wager (id, user_id, stake, price, created_at) VALUES ($1, $2, $3, $4, $5)",
            &[&wager.id, &wager.user_id, &stake, &price, &wager.created_at],
        )
        .await?;
    for draw_id in &draws {
        client
            .execute(
                "INSERT INTO draw_wager (draw_id ,wager_id) VALUES ($1, $2)",
                &[&draw_id, &wager.id],
            )
            .await?;
    }
    for board in &wager.boards{
      client
          .execute(
              "INSERT INTO board (id, wager_id, game_type) VALUES ($1, $2, $3)",
              &[&board.id, &board.wager_id, &board.game_type.to_string()],
          )
          .await?;
      for selection in &board.selections {
          client
              .execute(
                 "INSERT INTO selection (id, board_id, name, values) VALUES ($1, $2, $3, $4)",
                  &[&selection.id, &board.id, &selection.name, &selection.values],
          )
          .await?;
      }
    }
    info!("Successfully inserted wager: {:?} to draws {:?}", wager, draws);
    Ok(())
}

pub async fn insert_board(client: &Client, board: &Board) -> Result<(), Error> {
    info!("Attempting to insert board: {:?}", board);
    client
        .execute(
            "INSERT INTO board (id, wager_id, game_type) VALUES ($1, $2, $3)",
            &[&board.id, &board.wager_id, &board.game_type.to_string()],
        )
        .await?;
    info!("Successfully inserted board: {:?}", board);
    Ok(())
}

pub async fn insert_selection(client: &Client, selection: &Selection, board_id: Uuid) -> Result<(), Error> {
    info!("Attempting to insert selection: {:?}", selection);
    client
        .execute(
            "INSERT INTO selection (id, board_id, name, values) VALUES ($1, $2, $3, $4)",
            &[&selection.id, &board_id, &selection.name, &selection.values],
        )
        .await?;
    info!("Successfully inserted selection: {:?}", selection);
    Ok(())
}
