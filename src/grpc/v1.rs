use tonic::{Request, Response, Status};
use tracing::{instrument, trace};

use runners::runners_server::{Runners, RunnersServer};
use runners::AnnounceRequest;

pub mod runners {
    tonic::include_proto!("webalert.runners.v1");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("runners_descriptor");
}

#[derive(Default)]
pub struct RunnersService;

#[tonic::async_trait]
impl Runners for RunnersService {
    #[instrument(skip(request, self))]
    async fn announce(&self, request: Request<AnnounceRequest>) -> Result<Response<()>, Status> {
        let announce_req = request.into_inner();
        trace!(%announce_req.os,
            %announce_req.hostname,
            %announce_req.arch,
            "Received runner announcement");

        Ok(Response::new(()))
    }
}

/// Returns a list of reflection descriptor sets for this api.
pub fn file_descriptor_sets<'a>() -> Vec<&'a [u8]> {
    vec![runners::FILE_DESCRIPTOR_SET]
}

/// Creates and returns the gRPC `Runners` service.
pub fn create_runners_service() -> RunnersServer<RunnersService> {
    let runners = RunnersService::default();

    RunnersServer::new(runners)
}
