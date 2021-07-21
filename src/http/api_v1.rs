use std::convert::Infallible;

use chrono::{DateTime, Utc};
use log::debug;
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
pub struct CreateAlertRequest {
    url: String,
    selector: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAlertResponse {
    id: u64,
    status: String,
}

#[derive(Debug, Serialize)]
struct ListAlertsResponse {
    status: String,
    alerts: Vec<Alert>,
    num_alerts: u64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Runner {
    id: i32,
    name: String,
    hostname: String,
    arch: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunnerRequest {
    name: String,
    hostname: String,
    arch: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRunnerResponse {
    id: u64,
    status: String,
}

#[derive(Debug, Serialize)]
struct ListRunnersResponse {
    status: String,
    runners: Vec<Runner>,
    num_runners: u64,
}

mod handlers {
    use super::*;
    use crate::database::DbPool;

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

        let json = warp::reply::json(&ListAlertsResponse {
            status: "ok".to_owned(),
            num_alerts: alerts.len() as u64,
            alerts,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    /// Creates a new alert
    pub async fn create_alert(
        db: DbPool,
        request: CreateAlertRequest,
        token: Token,
    ) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        let res: (i32,) = sqlx::query_as(
            "INSERT INTO alerts (url, selector, creator_token) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(request.url)
        .bind(request.selector)
        .bind(token)
        .fetch_one(&db)
        .await
        .map_err(|_| reject::not_found())?;

        let json = warp::reply::json(&CreateAlertResponse {
            status: "ok".to_owned(),
            id: res.0 as u64,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    /// Returns a list of registered runners
    pub async fn list_runners(db: DbPool, token: Token) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        let runners = sqlx::query_as("SELECT * FROM runners").fetch_all(&db).await;

        let runners = match runners.ok() {
            Some(runners) => runners,
            None => vec![],
        };

        let json = warp::reply::json(&ListRunnersResponse {
            status: "ok".to_owned(),
            num_runners: runners.len() as u64,
            runners,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    /// Creates a new runner
    pub async fn create_runner(
        db: DbPool,
        token: Token,
        request: CreateRunnerRequest,
    ) -> Result<impl Reply, Rejection> {
        validate_token(db.clone(), &token).await?;

        let res: (i32,) = sqlx::query_as(
            "INSERT INTO runners (name, hostname, arch) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(request.name)
        .bind(request.hostname)
        .bind(request.arch)
        .fetch_one(&db)
        .await
        .map_err(|_| reject::custom(TokenRejection))?;

        let json = warp::reply::json(&CreateRunnerResponse {
            status: "ok".to_owned(),
            id: res.0 as u64,
        });

        Ok(warp::reply::with_status(json, StatusCode::OK))
    }

    /// Queries the database for the existance of the given `token` and returns it, unless it
    /// doesn't exist in which case a `Rejection` error is returned
    async fn validate_token(db: DbPool, token: &str) -> Result<Token, Rejection> {
        let row = sqlx::query_as("SELECT token FROM tokens WHERE token = $1::TEXT")
            .bind(token)
            .fetch_optional(&db)
            .await
            .map_err(|_| reject::custom(TokenRejection))?;

        let token: (String,) = row.ok_or_else(|| reject::custom(TokenRejection))?;

        Ok(token.0)
    }
}

mod filters {
    use super::{handlers, CreateAlertRequest, CreateRunnerRequest, Token};
    use crate::database::DbPool;
    use warp::Filter;

    // GET /…/_internal/runners
    pub fn runners(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        runners_list(db.clone()).or(runners_create(db))
    }

    // GET /…/runners
    pub fn runners_list(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("runners")
            .and(warp::get())
            .and(with_db(db))
            .and(warp::header::<Token>("x-token"))
            .and_then(handlers::list_runners)
    }

    // POST /…/runners
    pub fn runners_create(
        db: DbPool,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("runners")
            .and(warp::post())
            .and(with_db(db))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::header::<Token>("x-token"))
            .and(warp::body::json::<CreateRunnerRequest>())
            .and_then(handlers::create_runner)
    }

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
            .and(warp::body::json::<CreateAlertRequest>())
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
    use warp::filters::body::BodyDeserializeError;
    use warp::reject::*;

    let code;
    let message;

    debug!("Returning error: {:?}", err);

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if err.find::<BodyDeserializeError>().is_some() {
        code = StatusCode::UNPROCESSABLE_ENTITY;
        message = "INVALID_BODY";
    } else if err.find::<UnsupportedMediaType>().is_some() {
        code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
        message = "INVALID_CONTENT_TYPE";
    } else if err.find::<MethodNotAllowed>().is_some() {
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
pub use filters::runners;
