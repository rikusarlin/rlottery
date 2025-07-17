use tonic::{Request, Response, Status};
use prost_types;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::Mutex;
use tokio_postgres::Client;
use crate::db;
use crate::core::board::{GameType};
use crate::core::draw::Draw;
use tracing::{info, error};
use chrono::Utc;

pub mod wagering {
    tonic::include_proto!("wagering");
}

use wagering::{
    PlaceWagerRequest,
    PlaceWagerResponse,
    GetWagerRequest,
    GetWagerResponse,
};

pub struct WageringService{
  client: Arc<Mutex<Client>>,
}

impl WageringService {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        WageringService { client }
    }
}

#[tonic::async_trait]
impl wagering::wagering_server::Wagering for WageringService {
    async fn place_wager(
        &self,
        request: Request<PlaceWagerRequest>,
    ) -> Result<Response<PlaceWagerResponse>, Status> {
        info!("Got a request: {:?}", request);

        let client_locked = self.client.lock().await;

        let request_data = request.into_inner();
        let user_id = request_data.user_id.unwrap_or_default().value;
        let boards_proto = request_data.boards;
        let _quick_pick = request_data.quick_pick;

        

        // Get open draws
        let open_draws = db::draw::get_active_draws(&client_locked, uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-1234-567890abcdef").unwrap())
            .await
            .map_err(|e| {
                error!("Failed to get open draws: {}", e);
                Status::internal(format!("Failed to get open draws: {}", e))
            })?;

        if open_draws.is_empty() {
            return Err(Status::failed_precondition("No open draws available to place wager"));
        }

        if request_data.draws.is_empty() {
            return Err(Status::invalid_argument("No draw IDs provided in request"));
        }

        // Validate that all requested draws are open
        let open_draw_ids: HashSet<i32> = open_draws.iter().map(|d| d.id).collect();
        for draw_id in &request_data.draws {
            if !open_draw_ids.contains(draw_id) {
                return Err(Status::failed_precondition(format!(
                    "Requested draw {} is not currently open",
                    draw_id
                )));
            }
        }

        let selected_draws: Vec<Draw> = open_draws
            .into_iter()
            .filter(|d| request_data.draws.contains(&d.id))
            .collect();

        let wager_id = uuid::Uuid::new_v4();
        let user_uuid = uuid::Uuid::parse_str(&user_id).unwrap_or_default();

        let mut boards = Vec::new();
        for board_proto in &boards_proto {
            let board_id = uuid::Uuid::new_v4();
            let mut selections = Vec::new();
            for selection_proto in &board_proto.selections {
                let selection_id = uuid::Uuid::new_v4();
                let draw_level_uuid = uuid::Uuid::parse_str(&selection_proto.draw_level_id.clone().unwrap_or_default().value).unwrap_or_default();
                selections.push(crate::core::selection::Selection {
                    id: selection_id,
                    draw_level_id: draw_level_uuid,
                    values: selection_proto.values.clone(),
                });
            }
            let game_type_proto = &board_proto.game_type;
            let system_game_level_proto = &board_proto.system_game_level;
            let game_type = match game_type_proto {
                0 => GameType::NORMAL,
                1 => GameType::SYSTEM,
                _ => return Err(Status::invalid_argument("Invalid game type")),
            };
            let new_board = crate::core::board::Board {
                id: board_id,
                wager_id: wager_id,
                game_type: game_type.clone(),
                system_game_level: *system_game_level_proto,
                selections: selections,
            };
            boards.push(new_board);
        }

        let new_wager = crate::core::wager::Wager {
            id: wager_id,
            user_id: user_uuid,
            draws: selected_draws.clone(),
            boards: boards.clone(),
            stake: 100, // TODO: Calculate actual stake based on board(s)
            price: 100 * selected_draws.len() as u32, // TODO: Calculate actual price based on stake
            created_at: Utc::now(),
        };
        db::wager::insert_wager(&client_locked, &new_wager, request_data.draws)
            .await
            .map_err(|e| {
                error!("Failed to insert wager: {}", e);
                Status::internal(format!("Failed to insert wager: {}", e))
            })?;

        let proto_boards: Vec<wagering::Board> = new_wager.boards.into_iter().map(|board| {
            let proto_selections = board.selections.into_iter().map(|selection| {
                wagering::Selection {
                    id: Some(wagering::Uuid { value: selection.id.to_string() }),
                    draw_level_id: Some(wagering::Uuid { value: selection.draw_level_id.to_string() }),
                    values: selection.values,
                }
            }).collect();

            let game_type = match board.game_type {
                GameType::NORMAL => wagering::GameType::Normal,
                GameType::SYSTEM => wagering::GameType::System,
            };

            wagering::Board {
                id: Some(wagering::Uuid { value: board.id.to_string() }),
                game_type: game_type.into(),
                system_game_level: board.system_game_level,
                selections: proto_selections,
            }
        }).collect();

        let reply = PlaceWagerResponse {
            wager: Some(wagering::Wager {
                id: Some(wagering::Uuid { value: new_wager.id.to_string() }),
                user_id: Some(wagering::Uuid { value: new_wager.user_id.to_string() }),
                draws: new_wager.draws.into_iter().map(|d| wagering::Draw {
                    id: d.id,
                    game_id: Some(wagering::Uuid { value: d.game_id.to_string() }),
                    status: d.status as i32,
                    created_at: Some(prost_types::Timestamp {
                        seconds: d.created_at.timestamp(),
                        nanos: d.created_at.timestamp_subsec_nanos() as i32,
                    }),
                    modified_at: Some(prost_types::Timestamp {
                        seconds: d.modified_at.timestamp(),
                        nanos: d.modified_at.timestamp_subsec_nanos() as i32,
                    }),
                    open_time: Some(prost_types::Timestamp {
                        seconds: d.open_time.timestamp(),
                        nanos: d.open_time.timestamp_subsec_nanos() as i32,
                    }),
                    close_time: Some(prost_types::Timestamp {
                        seconds: d.close_time.timestamp(),
                        nanos: d.close_time.timestamp_subsec_nanos() as i32,
                    }),
                    draw_time: d.draw_time.map(|t| prost_types::Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                    winset_calculated_at: d.winset_calculated_at.map(|t| prost_types::Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                    winset_confirmed_at: d.winset_confirmed_at.map(|t| prost_types::Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                    winning_numbers: vec![], // TODO: Populate winning numbers
                }).collect(),
                stake: new_wager.stake,
                price: new_wager.price,
                boards: proto_boards,
                created_at: Some(prost_types::Timestamp {
                    seconds: new_wager.created_at.timestamp(),
                    nanos: new_wager.created_at.timestamp_subsec_nanos() as i32,
                }),
            }),
        };
        info!("Returning PlaceWagerResponse: {:?}", reply);
        Ok(Response::new(reply))
    }

    async fn get_wager(
        &self,
        request: Request<GetWagerRequest>,
    ) -> Result<Response<GetWagerResponse>, Status> {
        info!("Got a GetWagerRequest: {:?}", request);

        let reply = GetWagerResponse {
            wager: None, // TODO: Implement actual wager retrieval logic
        };
        info!("Returning GetWagerResponse: {:?}", reply);
        Ok(Response::new(reply))
    }
}