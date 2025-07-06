use tonic::{Request, Response, Status};
use prost_types;

pub mod wagering {
    tonic::include_proto!("wagering");
}

use wagering::{
    wagering_server::Wagering,
    PlaceWagerRequest,
    PlaceWagerResponse,
    GetWagerRequest,
    GetWagerResponse,
};

#[derive(Debug, Default)]
pub struct WageringService;

#[tonic::async_trait]
impl Wagering for WageringService {
    async fn place_wager(
        &self,
        request: Request<PlaceWagerRequest>,
    ) -> Result<Response<PlaceWagerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let request_data = request.into_inner();
        let user_id = request_data.user_id.unwrap_or_default().value;
        let number_of_draws = request_data.number_of_draws;
        let boards_proto = request_data.boards;
        let _quick_pick = request_data.quick_pick;
        let game_type_proto = request_data.game_type;
        let system_game_level_proto = request_data.system_game_level;

        let mut created_wagers = Vec::new();

        let game_type = match game_type_proto {
            0 => crate::core::wager::GameType::NORMAL,
            1 => crate::core::wager::GameType::SYSTEM,
            _ => return Err(Status::invalid_argument("Invalid game type")),
        };

        for _ in 0..number_of_draws {
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
                        value: selection_proto.value,
                    });
                }
                boards.push(crate::core::board::Board {
                    id: board_id,
                    wager_id: wager_id,
                    selections: selections,
                });
            }

            let new_wager = crate::core::wager::Wager {
                id: wager_id,
                user_id: user_uuid,
                draw_id: 0, // TODO: Get actual draw_id
                game_type: game_type.clone(),
                system_game_level: system_game_level_proto,
                stake: 1.0, // TODO: Calculate actual stake
                price: 1.0 * number_of_draws as f64, // TODO: Calculate actual price
                created_at: chrono::Utc::now(),
            };
            created_wagers.push(new_wager);
        }

        let reply = PlaceWagerResponse {
            wagers: created_wagers.into_iter().map(|w| wagering::Wager {
                id: Some(wagering::Uuid { value: w.id.to_string() }),
                user_id: Some(wagering::Uuid { value: w.user_id.to_string() }),
                draw_id: w.draw_id,
                game_type: match w.game_type {
                    crate::core::wager::GameType::NORMAL => wagering::GameType::Normal as i32,
                    crate::core::wager::GameType::SYSTEM => wagering::GameType::System as i32,
                },
                system_game_level: w.system_game_level,
                stake: w.stake,
                price: w.price,
                created_at: Some(prost_types::Timestamp {
                    seconds: w.created_at.timestamp(),
                    nanos: w.created_at.timestamp_subsec_nanos() as i32,
                }),
            }).collect(),
        };
        Ok(Response::new(reply))
    }

    async fn get_wager(
        &self,
        request: Request<GetWagerRequest>,
    ) -> Result<Response<GetWagerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = GetWagerResponse {
            wager: None, // TODO: Implement actual wager retrieval logic
        };
        Ok(Response::new(reply))
    }
}
