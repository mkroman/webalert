use std::convert::Infallible;

use chrono::{DateTime, Utc};
use serde::Serialize;
use warp::http::StatusCode;
use warp::{reject, reject::Reject, Rejection, Reply};

pub type Token = String;

#[derive(Debug)]
pub struct TokenRejection;

impl Reject for TokenRejection {}

#[derive(Serialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct Alert {
    id: i32,
    url: String,
    selector: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct AlertListResponse {
    status: String,
    alerts: Vec<Alert>,
    num_alerts: u64,
}

mod handlers {
    use super::*;
    use std::sync::Arc;
    use tokio_postgres::Client;

    /// Returns a JSON-serialized `AlertListResponse` with alerts beloning to the given `token`
    pub async fn list_alerts(db: Arc<Client>, token: Token) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        let alerts: Vec<Alert> = db
            .query(
                "SELECT * FROM alerts WHERE creator_token = $1::TEXT",
                &[&token],
            )
            .await
            .map_err(|_| reject::reject())?
            .iter()
            .map(|row| Alert {
                id: row.get(0),
                url: row.get(1),
                selector: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            })
            .collect();

        let json = warp::reply::json(&AlertListResponse {
            status: "ok".to_owned(),
            num_alerts: alerts.len() as u64,
            alerts,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    async fn validate_token(db: Arc<Client>, token: &Token) -> Result<Token, Rejection> {
        match db
            .query_one("SELECT token FROM tokens WHERE token = $1::TEXT", &[token])
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
