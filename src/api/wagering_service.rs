use tonic::{Request, Response, Status};

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

        let reply = PlaceWagerResponse {
            wager: None, // TODO: Implement actual wager placement logic
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
