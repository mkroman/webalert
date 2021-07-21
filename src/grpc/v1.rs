use tonic::{Request, Response, Status};

use runners::runners_server::{Runners, RunnersServer};
use runners::AnnounceRequest;

pub mod runners {
    tonic::include_proto!("webalert.runners.v1");
}

#[derive(Default)]
pub struct RunnersService;

#[tonic::async_trait]
impl Runners for RunnersService {
    async fn announce(&self, _request: Request<AnnounceRequest>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

pub fn create_runners_service() -> RunnersServer<RunnersService> {
    let runners = RunnersService::default();

    RunnersServer::new(runners)
}
