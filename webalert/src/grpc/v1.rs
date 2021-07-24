use tonic::{Request, Response, Status};
use tracing::{instrument, trace};

use runners::runner_server::{Runner, RunnerServer};
use runners::AnnounceRequest;

pub mod runners {
    tonic::include_proto!("webalert.runner.v1");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("runners_descriptor");
}

#[derive(Default, Debug)]
pub struct RunnerService;

#[tonic::async_trait]
impl Runner for RunnerService {
    #[instrument]
    async fn announce(&self, request: Request<AnnounceRequest>) -> Result<Response<()>, Status> {
        let announce_req = request.into_inner();

        trace!(
            %announce_req.os,
            %announce_req.hostname,
            %announce_req.arch,
            "Received runner announcement");

        Ok(Response::new(()))
    }
}

/// Returns a list of reflection descriptor sets for this api.
pub(crate) fn file_descriptor_sets<'a>() -> Vec<&'a [u8]> {
    vec![runners::FILE_DESCRIPTOR_SET]
}

/// Creates and returns the gRPC `Runners` service.
pub(crate) fn create_runners_service() -> RunnerServer<RunnerService> {
    let runner_svc = RunnerService::default();

    RunnerServer::new(runner_svc)
}