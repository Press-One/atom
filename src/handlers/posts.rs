use actix_web::{web, HttpResponse};

use super::Pagination;
use crate::db;
use crate::db::PgPool;
use crate::handlers::pg_pool_handler;
use crate::processor;
use serde::Serialize;

#[derive(Serialize)]
struct PostItem {
    pub publish_tx_id: String,
    pub file_hash: String,
    pub topic: String,
    pub updated_tx_id: String,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted: bool,
    pub content: String,
}

pub fn list_all_asc(pool: web::Data<PgPool>, params: web::Query<Pagination>) -> HttpResponse {
    // return json data to post web site
    let offset = params.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(params.limit.unwrap_or(20), 100) as i64;
    let topic = &params.topic;

    let db_conn_res = pg_pool_handler(pool);
    if let Ok(db_conn) = db_conn_res {
        let posts_result = db::get_posts_for_json(&db_conn, topic, offset, limit);
        match posts_result {
            Ok(posts) => {
                let mut post_vec: Vec<PostItem> = Vec::new();

                for p in posts {
                    let content_res = db::get_content(&db_conn, &p.file_hash);
                    match content_res {
                        Ok(content) => {
                            let item = PostItem {
                                publish_tx_id: p.publish_tx_id,
                                file_hash: p.file_hash,
                                topic: p.topic,
                                updated_at: p.updated_at,
                                updated_tx_id: p.updated_tx_id.trim().to_string(),
                                deleted: p.deleted,
                                content: content.content,
                            };
                            post_vec.push(item);
                        }
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(e.to_string());
                        }
                    }
                }
                HttpResponse::Ok().json(post_vec)
            }
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        }
    } else {
        HttpResponse::InternalServerError().json("connect to database failed")
    }
}

pub fn list_all_atom_by_asc(pool: web::Data<PgPool>, params: web::Query<Pagination>) -> String {
    let offset = params.offset.unwrap_or(0) as i64;
    let limit = std::cmp::min(params.limit.unwrap_or(20), 100) as i64;
    let topic = &params.topic;

    let db_conn_res = pg_pool_handler(pool);
    if let Ok(db_conn) = db_conn_res {
        let posts_result = db::get_all_atom_posts_by_asc(&db_conn, topic, offset, limit);
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
