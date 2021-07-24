use tonic::{Request, Response, Status};
use tracing::{instrument, trace};

use runners::runner_server::{Runner, RunnerServer};
use runners::{AnnounceRequest, ListRequest, ListResponse};

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
        let _announce_req = request.into_inner();

        trace!("Received runner announcement");

        Ok(Response::new(()))
    }

    #[instrument]
    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let _list_req = request.into_inner();

        trace!("Received runner list request");

        Err(Status::unimplemented("This method is not implemented yet"))
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
