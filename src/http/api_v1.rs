use std::convert::Infallible;

use serde::Serialize;
use warp::http::StatusCode;
use warp::{reject::Reject, Rejection, Reply};

pub type Token = String;

#[derive(Debug)]
pub struct TokenRejection;

impl Reject for TokenRejection {}

#[derive(Serialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
}

mod handlers {
    use super::{Token, TokenRejection};
    use std::sync::Arc;
    use tokio_postgres::Client;
    use warp::{reject, Rejection, Reply};

    pub async fn list_alerts(db: Arc<Client>, token: Token) -> Result<impl Reply, Rejection> {
        match db
            .query_one("SELECT token FROM tokens WHERE token = $1::TEXT", &[&token])
            .await
        {
            Ok(row) => Ok(row.get::<_, Token>(0)),
            Err(_) => Err(reject::custom(TokenRejection)),
        }
    }
}

mod filters {
    use super::{handlers, Token};
    use std::sync::Arc;
    use tokio_postgres::Client;
    use warp::Filter;

    // GET /alerts/… with an `X-TOKEN` header
    pub fn alerts(
        db: Arc<Client>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        alerts_list(db)
    }

    // GET /…/alerts
    pub fn alerts_list(
        db: Arc<Client>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("alerts")
            .and(warp::get())
            .and(with_db(db))
            .and(warp::header::<Token>("x-token"))
            .and_then(handlers::list_alerts)
    }

    fn with_db(
        db: Arc<Client>,
    ) -> impl Filter<Extract = (Arc<Client>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else if let Some(TokenRejection) = err.find() {
        code = StatusCode::FORBIDDEN;
        message = "INVALID_TOKEN";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.to_string(),
    });

    Ok(warp::reply::with_status(json, code))
}

pub use filters::alerts;
