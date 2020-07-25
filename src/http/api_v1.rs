use std::convert::Infallible;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;
use warp::{
    reject::{self, Reject},
    Rejection, Reply,
};

pub type Token = String;

#[derive(Debug)]
pub struct TokenRejection;

impl Reject for TokenRejection {}

#[derive(Serialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Alert {
    id: i32,
    url: String,
    selector: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AlertCreateRequest {
    url: String,
    selector: String,
}

#[derive(Debug, Serialize)]
struct AlertListResponse {
    status: String,
    alerts: Vec<Alert>,
    num_alerts: u64,
}

mod handlers {
    use super::*;
    use crate::database::DbPool;
    use sqlx::prelude::*;

    /// Returns a JSON-serialized `AlertListResponse` with alerts belonging to the given `token`
    pub async fn list_alerts(db: DbPool, token: Token) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        let alerts = match sqlx::query_as("SELECT * FROM alerts WHERE creator_token = $1::TEXT")
            .bind(&token)
            .fetch_all(&db)
            .await
            .ok()
        {
            Some(alerts) => alerts,
            None => vec![],
        };

        let json = warp::reply::json(&AlertListResponse {
            status: "ok".to_owned(),
            num_alerts: alerts.len() as u64,
            alerts,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    /// Creates a new alert
    pub async fn create_alert(
        db: DbPool,
        request: AlertCreateRequest,
        token: Token,
    ) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        sqlx::query("INSERT INTO alerts (url, selector, creator_token) VALUES ($1, $2, $3)")
            .bind(request.url)
            .bind(request.selector)
            .bind(token)
            .execute(&db)
            .await
            .map_err(|_| reject::custom(TokenRejection))?;

        Ok(warp::reply::with_status(":)", StatusCode::OK))
    }

    /// Queries the database for the existance of the given `token` and returns it, unless it
    /// doesn't exist in which case a `Rejection` error is returned
    async fn validate_token(db: DbPool, token: &Token) -> Result<Token, Rejection> {
        let result: Option<String> =
            sqlx::query_as("SELECT token FROM tokens WHERE token = $1::TEXT")
                .bind(token)
                .fetch_optional(&db)
                .await
                .map_err(|_| reject::custom(TokenRejection))?
                .map(|row: (String,)| row.0);

        match result {
            Some(token) => Ok(token),
            None => Err(reject::custom(TokenRejection)),
        }
    }
}

mod filters {
    use super::{handlers, AlertCreateRequest, Token};
    use crate::database::DbPool;
    use warp::Filter;

    // GET /alerts/… with an `X-TOKEN` header
    pub fn alerts(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        alerts_list(db.clone()).or(alerts_create(db))
    }

    // GET /…/alerts
    pub fn alerts_list(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("alerts")
            .and(warp::get())
            .and(with_db(db))
            .and(warp::header::<Token>("x-token"))
            .and_then(handlers::list_alerts)
    }

    // POST /alerts
    pub fn alerts_create(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("alerts")
            .and(warp::post())
            .and(with_db(db))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json::<AlertCreateRequest>())
            .and(warp::header::<Token>("x-token"))
            .and_then(handlers::create_alert)
    }

    fn with_db(
        db: DbPool,
    ) -> impl Filter<Extract = (DbPool,), Error = std::convert::Infallible> + Clone {
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
