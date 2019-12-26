use actix_web::{web, HttpResponse, Result};

use crate::db::models;
use crate::db::PgPool;
use crate::handlers::pg_pool_handler;

use super::Pagination;

pub fn list(pool: web::Data<PgPool>, pagination: web::Query<Pagination>) -> Result<HttpResponse> {
    let offset = pagination.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(pagination.limit.unwrap_or(20), 100) as i64;

    let db_conn = pg_pool_handler(pool)?;
    Ok(HttpResponse::Ok().json(models::BlockList::list(&db_conn, offset, limit)))
}
