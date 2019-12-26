use actix_web::web;

use super::Pagination;
use crate::db;
use crate::db::PgPool;
use crate::handlers::pg_pool_handler;
use crate::processor;

pub fn list_all_asc(pool: web::Data<PgPool>, params: web::Query<Pagination>) -> String {
    let offset = params.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(params.limit.unwrap_or(20), 100) as i64;
    let topic = &params.topic;

    let db_conn_res = pg_pool_handler(pool);
    if let Ok(db_conn) = db_conn_res {
        let posts_result = db::get_all_posts_by_page(&db_conn, topic, offset, limit);
        match posts_result {
            Ok(posts) => {
                let atomstring = processor::atom(&db_conn, posts);
                atomstring
            }
            Err(e) => String::from(format!("{}", e)),
        }
    } else {
        String::from("connect to database failed")
    }
}

pub fn list_latest(pool: web::Data<PgPool>, params: web::Query<Pagination>) -> String {
    let offset = params.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(params.limit.unwrap_or(20), 100) as i64;
    let topic = &params.topic;

    let db_conn_res = pg_pool_handler(pool);
    if let Ok(db_conn) = db_conn_res {
        let posts_result = db::get_latest_posts_by_page(&db_conn, topic, offset, limit);
        match posts_result {
            Ok(posts) => {
                let atomstring = processor::atom(&db_conn, posts);
                atomstring
            }
            Err(e) => String::from(format!("{}", e)),
        }
    } else {
        String::from("connect to database failed")
    }
}
