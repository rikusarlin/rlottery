use tonic::{Request, Response, Status};
use prost_types::Timestamp;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;
use tracing::{info, error};
use chrono::{DateTime, Utc};
use crate::api::wagering_service::wagering;

pub mod draw {
    tonic::include_proto!("draw");
}

pub struct DrawService {
    client: Arc<Mutex<Client>>
}

impl DrawService {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        DrawService { client }
    }
}

#[tonic::async_trait]
impl draw::draw_service_server::DrawService for DrawService {
    async fn get_open_draws(
        &self,
        request: Request<draw::GetOpenDrawsRequest>,
    ) -> Result<Response<draw::GetOpenDrawsResponse>, Status> {
        info!("Got a request: {:?}", request);
        let client_locked = self.client.lock().await;

        let game_id_option = request.into_inner().game_id;
        let mut query = "SELECT id, game_id, status, created_at, modified_at, open_time, close_time, draw_time, winset_calculated_at, winset_confirmed_at FROM draw WHERE status = 'Open'".to_string();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

        if let Some(game_id_proto) = &game_id_option {
            query.push_str(" AND game_id = $1");
            let game_uuid = Uuid::parse_str(&game_id_proto.value).map_err(|e| Status::invalid_argument(format!("Invalid game_id UUID: {}", e)))?;
            params.push(Box::new(game_uuid));
        }

        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
        let rows = client_locked.query(&query, params_refs.as_slice()).await.map_err(|e| {
            error!("Failed to fetch open draws: {}", e);
            Status::internal(format!("Failed to fetch open draws: {}", e))
        })?;

        let draws: Vec<wagering::Draw> = rows.into_iter().map(|row| {
            let draw_time: Option<DateTime<Utc>> = row.get("draw_time");
            let game_id_uuid: Uuid = row.get("game_id");
            wagering::Draw {
                id: row.get("id"),
                game_id: Some(wagering::Uuid { value: game_id_uuid.to_string() }),
                status: match row.get::<_, String>("status").as_str() {
                    "Created" => wagering::DrawStatus::Created.into(),
                    "Open" => wagering::DrawStatus::Open.into(),
                    "Closed" => wagering::DrawStatus::Closed.into(),
                    "Drawn" => wagering::DrawStatus::Drawn.into(),
                    "WinsetCalculated" => wagering::DrawStatus::WinsetCalculated.into(),
                    "WinsetConfirmed" => wagering::DrawStatus::WinsetConfirmed.into(),
                    "Finalized" => wagering::DrawStatus::Finalized.into(),
                    "Cancelled" => wagering::DrawStatus::Cancelled.into(),
                    _ => 0, // Default to 0 for unspecified
                },
                created_at: Some(Timestamp { seconds: row.get::<_, DateTime<Utc>>("created_at").timestamp(), nanos: row.get::<_, DateTime<Utc>>("created_at").timestamp_subsec_nanos() as i32 }),
                modified_at: Some(Timestamp { seconds: row.get::<_, DateTime<Utc>>("modified_at").timestamp(), nanos: row.get::<_, DateTime<Utc>>("modified_at").timestamp_subsec_nanos() as i32 }),
                open_time: Some(Timestamp { seconds: row.get::<_, DateTime<Utc>>("open_time").timestamp(), nanos: row.get::<_, DateTime<Utc>>("open_time").timestamp_subsec_nanos() as i32 }),
                close_time: Some(Timestamp { seconds: row.get::<_, DateTime<Utc>>("close_time").timestamp(), nanos: row.get::<_, DateTime<Utc>>("close_time").timestamp_subsec_nanos() as i32 }),
                draw_time: draw_time.map(|dt| Timestamp { seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32 }),
                winset_calculated_at: row.get::<_, Option<DateTime<Utc>>>("winset_calculated_at").map(|dt| Timestamp { seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32 }),
                winset_confirmed_at: row.get::<_, Option<DateTime<Utc>>>("winset_confirmed_at").map(|dt| Timestamp { seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32 }),
                winning_numbers: Vec::new(),
            }
        }).collect();

        let reply = draw::GetOpenDrawsResponse { draws };
        Ok(Response::new(reply))
    }
}