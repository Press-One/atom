use super::db::{PgPool, PgPooledConnection};
use actix_web::web;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

pub mod posts;
pub mod users;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub topic: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

pub fn pg_pool_handler(pool: web::Data<PgPool>) -> Result<PgPooledConnection, HttpResponse> {
    pool.get()
        .map_err(|e| HttpResponse::InternalServerError().json(e.to_string()))
}
