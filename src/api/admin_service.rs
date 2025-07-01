use tonic::{Request, Response, Status};

pub mod admin {
    tonic::include_proto!("admin");
}

use admin::{admin_server::Admin, ReceiveExternalDrawNumbersRequest, ReceiveExternalDrawNumbersResponse};

#[derive(Debug, Default)]
pub struct AdminService;

#[tonic::async_trait]
impl Admin for AdminService {
    async fn receive_external_draw_numbers(
        &self,
        request: Request<ReceiveExternalDrawNumbersRequest>,
    ) -> Result<Response<ReceiveExternalDrawNumbersResponse>, Status> {
        println!("Received external draw numbers: {:?}", request);

        // TODO: Implement logic to update the draw with external numbers

        let reply = ReceiveExternalDrawNumbersResponse {
            success: true,
            message: "External draw numbers received successfully.".to_string(),
        };
        Ok(Response::new(reply))
    }
}
