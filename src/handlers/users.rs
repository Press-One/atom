use actix_web::{web, HttpResponse, Result};

use super::Pagination;
use crate::db::models;
use crate::db::PgPool;
use crate::handlers::pg_pool_handler;

pub fn list(pool: web::Data<PgPool>, pagination: web::Query<Pagination>) -> Result<HttpResponse> {
    let topic = &pagination.topic;
    let offset = pagination.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(pagination.limit.unwrap_or(20), 100) as i64;

    let db_conn = pg_pool_handler(pool)?;
    Ok(HttpResponse::Ok().json(models::UserList::list(&db_conn, topic, offset, limit)))
}
