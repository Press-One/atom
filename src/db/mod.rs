extern crate chrono;

use chrono::prelude::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use std::time::Duration;

pub mod models;
pub mod schema;
use super::prs;

use self::models::{Content, NewContent};
use self::models::{LastStatus, NewLastStatus};
use self::models::{NewNotify, Notify, NotifyPartial};
use self::models::{NewPost, Post, PostJson, PostPartial};
use self::models::{NewTrx, Trx};
use self::models::{NewUser, User, UserList};
use super::SETTINGS;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let pool_size = 2; // FIXME: hardcode
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .connection_timeout(Duration::from_millis(10 * 1000))
        .max_size(pool_size)
        .test_on_check_out(true)
        .build(manager)
}

pub fn establish_connection_pool() -> PgPool {
    init_pool(&SETTINGS.atom.db_url).expect("create database pool failed")
}

pub fn save_user<'a>(
    conn: &PgConnection,
    user_address: &'a str,
    status: &'a str,
    tx_id: &'a str,
    topic: &'a str,
    updated_at: chrono::NaiveDateTime,
) -> Result<User, diesel::result::Error> {
    use schema::users;

    let new_user = NewUser {
        user_address,
        status,
        tx_id,
        updated_at,
        topic,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .on_conflict(users::user_address)
        .do_update()
        .set(&new_user)
        .get_result(conn)
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
pub fn save_post<'a>(
    conn: &PgConnection,
    publish_tx_id: &'a str,
    user_address: &'a str,
    updated_tx_id: &'a str,
    file_hash: &'a str,
    hash_alg: &'a str,
    topic: &'a str,
    url: &'a str,
    encryption: &str,
    updated_at: chrono::NaiveDateTime,
) -> Result<Post, diesel::result::Error> {
    use schema::posts;

    let new_post = NewPost {
        publish_tx_id,
        user_address,
        updated_tx_id,
        file_hash,
        topic,
        url,
        encryption,
        hash_alg,
        updated_at,
    };

    diesel::insert_into(posts::table)
        .values(&new_post)
        .on_conflict(posts::publish_tx_id)
        .do_update()
        .set(&new_post)
        .get_result(conn)
}

pub fn save_content<'a>(
    conn: &PgConnection,
    file_hash: &'a str,
    url: &'a str,
    content: &'a str,
) -> Result<Content, diesel::result::Error> {
    use schema::contents;
    let now = Utc::now().naive_utc();
    let new_content = NewContent {
        file_hash,
        url,
        content,
        created_at: now,
    };

    diesel::insert_into(contents::table)
        .values(&new_content)
        .get_result(conn)
}

pub fn get_post_by_publish_tx_id(
    conn: &PgConnection,
    publish_tx_id: &str,
) -> Result<Post, diesel::result::Error> {
    use schema::posts;

    posts::table
        .filter(posts::publish_tx_id.eq(publish_tx_id))
        .first::<Post>(conn)
}

pub fn get_posts(
    conn: &PgConnection,
    fetch_status: bool,
    limit: i64,
) -> Result<Vec<Post>, diesel::result::Error> {
    use schema::posts::dsl::*;
    posts
        .filter(fetched.eq(fetch_status))
        .filter(deleted.eq(false))
        .limit(limit)
        .load::<Post>(conn)
}

pub fn get_allow_posts(
    conn: &PgConnection,
    topic: &str,
) -> Result<Vec<PostPartial>, diesel::result::Error> {
    let sql = format!(
        r#"
        SELECT posts.publish_tx_id, posts.file_hash, posts.topic, posts.deleted
        FROM posts, users
        WHERE posts.user_address = users.user_address
        AND posts.topic = '{}'
        AND posts.deleted = 'f'
        AND posts.fetched = 't'
        AND posts.verify = 't'
        AND users.status = 'allow'
        ORDER BY posts.updated_at desc
        "#,
        topic
    );
    diesel::sql_query(sql).load::<PostPartial>(conn)
}

pub fn get_posts_for_json(
    conn: &PgConnection,
    topic: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<PostJson>, diesel::result::Error> {
    let sql = format!(
        r#"
        SELECT posts.publish_tx_id, posts.file_hash, posts.topic, posts.updated_tx_id, posts.updated_at, posts.deleted
        FROM posts, users
        WHERE posts.user_address = users.user_address
        AND posts.topic = '{}'
        AND posts.fetched = 't'
        AND posts.verify = 't'
        ORDER BY posts.updated_at asc
        OFFSET {}
        LIMIT {}
        "#,
        topic, offset, limit
    );
    diesel::sql_query(sql).load::<PostJson>(conn)
}

pub fn get_all_atom_posts_by_asc(
    conn: &PgConnection,
    topic: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<PostPartial>, diesel::result::Error> {
    let sql = format!(
        r#"
        SELECT posts.publish_tx_id, posts.file_hash, posts.topic, posts.deleted
        FROM posts, users
        WHERE posts.user_address = users.user_address
        AND posts.topic = '{}'
        AND posts.fetched = 't'
        AND posts.verify = 't'
        ORDER BY posts.updated_at asc
        OFFSET {}
        LIMIT {}
        "#,
        topic, offset, limit
    );
    diesel::sql_query(sql).load::<PostPartial>(conn)
}

pub fn get_latest_posts_by_page(
    conn: &PgConnection,
    topic: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<PostPartial>, diesel::result::Error> {
    let sql = format!(
        r#"
        SELECT posts.publish_tx_id, posts.file_hash, posts.topic, posts.deleted
        FROM posts, users
        WHERE posts.user_address = users.user_address
        AND posts.topic = '{}'
        AND posts.fetched = 't'
        AND posts.verify = 't'
        AND posts.deleted = 'f'
        AND users.status = 'allow'
        ORDER BY posts.updated_at desc
        OFFSET {}
        LIMIT {}
        "#,
        topic, offset, limit
    );
    diesel::sql_query(sql).load::<PostPartial>(conn)
}

pub fn get_content<'a>(
    conn: &PgConnection,
    file_hash: &'a str,
) -> Result<Content, diesel::result::Error> {
    use schema::contents;
    contents::table
        .find(file_hash)
        .first::<models::Content>(conn)
}

pub fn delete_content<'a>(
    conn: &PgConnection,
    _file_hash: &'a str,
) -> Result<usize, diesel::result::Error> {
    use schema::contents::dsl::*;

    diesel::update(contents.filter(file_hash.eq(_file_hash)))
        .set((deleted.eq(true), updated_at.eq(Utc::now().naive_utc())))
        .execute(conn)
}

pub fn delete_post<'a>(
    conn: &PgConnection,
    _file_hash: &'a str,
) -> Result<usize, diesel::result::Error> {
    use schema::posts::dsl::*;

    diesel::update(posts.filter(file_hash.eq(_file_hash)))
        .set((deleted.eq(true), updated_at.eq(Utc::now().naive_utc())))
        .execute(conn)
}

pub fn update_post_status<'a>(
    conn: &PgConnection,
    input_file_hash: &'a str,
    fetched_flag: bool,
    verify_flag: bool,
) -> Result<usize, diesel::result::Error> {
    use schema::posts::dsl::*;

    let result = diesel::update(posts.filter(file_hash.eq(input_file_hash)))
        .set((
            fetched.eq(fetched_flag),
            verify.eq(verify_flag),
            updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn);
    debug!(
        "update posts set fetched = {}, verify = {} where file_hash = {}",
        fetched_flag, verify_flag, input_file_hash
    );
    result
}

pub fn get_last_status(
    conn: &PgConnection,
    _key: &str,
) -> Result<LastStatus, diesel::result::Error> {
    use schema::last_status::dsl::*;
    last_status.filter(key.eq(_key)).first::<LastStatus>(conn)
}

pub fn get_max_tx_num(conn: &PgConnection) -> Result<i32, diesel::result::Error> {
    use schema::transactions::dsl::*;

    let result = transactions.order(id.desc()).first::<Trx>(conn);
    match result {
        Ok(tx) => {
            let tx_max_num = tx.id;
            Ok(tx_max_num)
        }
        Err(_) => Ok(0),
    }
}

pub fn update_last_status(
    conn: &PgConnection,
    _key: &str,
    _val: i64,
) -> Result<LastStatus, diesel::result::Error> {
    use schema::last_status;
    let result = get_last_status(conn, _key);
    match result {
        Ok(last) => {
            let id = last.id;
            diesel::update(last_status::table.filter(last_status::id.eq(id)))
                .set(last_status::val.eq(_val))
                .get_result::<LastStatus>(conn)
        }
        Err(e) => {
            if e == diesel::NotFound {
                let new_last = NewLastStatus {
                    key: _key,
                    val: _val,
                };
                diesel::insert_into(last_status::table)
                    .values(&new_last)
                    .get_result(conn)
            } else {
                error!("update last status table {}={} failed: {}", _key, _val, e);
                Err(e)
            }
        }
    }
}

pub fn save_trx(conn: &PgConnection, trx: &prs::Transaction) -> Result<Trx, diesel::result::Error> {
    use schema::transactions;

    let action_data: prs::Pip2001ActionData = trx.data.clone();
    let new_trx = NewTrx {
        block_num: trx.block_num,
        data_type: &action_data._type,
        data: &json!(action_data).to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
        trx_id: &trx.trx_id.clone(),
        signature: &action_data.signature,
        hash: &action_data.hash,
        user_address: &action_data.user_address,
    };

    let item = diesel::insert_into(transactions::table)
        .values(&new_trx)
        .on_conflict(transactions::trx_id)
        .do_update()
        .set(&new_trx)
        .get_result(conn);

    info!(
        "saved trx data from block_num = {}, data_type = {}",
        trx.block_num, trx.data_type
    );

    item
}

pub fn get_trx_by_trx_id(conn: &PgConnection, trx_id: &str) -> Result<Trx, diesel::result::Error> {
    use schema::transactions;

    transactions::table
        .filter(transactions::trx_id.eq(trx_id))
        .first::<Trx>(conn)
}

pub fn get_trxs(
    conn: &PgConnection,
    is_processed: bool,
) -> Result<Vec<Trx>, diesel::result::Error> {
    use schema::transactions::dsl::*;

    transactions
        .filter(processed.eq(is_processed))
        .order(block_num.asc())
        .load::<Trx>(conn)
}

pub fn update_trx_status(
    conn: &PgConnection,
    _block_num: i64,
    _processed: bool,
) -> Result<usize, diesel::result::Error> {
    use schema::transactions::dsl::*;

    let result = diesel::update(transactions.filter(block_num.eq(_block_num)))
        .set((
            processed.eq(_processed),
            updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn);
    debug!(
        "update transaction set processed = {} where block_num = {}",
        _processed, _block_num
    );
    result
}

pub fn save_notify(
    conn: &PgConnection,
    data_id: &str,
    block_num: i64,
    trx_id: &str,
    topic: &str,
) -> Result<Notify, diesel::result::Error> {
    use schema::notifies;

    let new_notify = NewNotify {
        data_id,
        block_num,
        trx_id,
        topic,
    };

    let notify = diesel::insert_into(notifies::table)
        .values(&new_notify)
        .on_conflict(notifies::data_id)
        .do_update()
        .set(&new_notify)
        .get_result(conn);

    info!(
        "saved notify data, data_id = {} block_num = {} trx_id = {}",
        data_id, block_num, trx_id
    );

    notify
}

pub fn get_unnotified_list(
    conn: &PgConnection,
) -> Result<Vec<NotifyPartial>, diesel::result::Error> {
    let sql = r#"
        SELECT
            notifies.data_id,
            notifies.block_num,
            notifies.trx_id,
            notifies.topic
        FROM notifies, posts
        WHERE
            notifies.success = 'f'
            and notifies.data_id = posts.publish_tx_id
            and posts.deleted = 'f'
            and posts.fetched = 't'
            and posts.verify = 't'
        "#;
    diesel::sql_query(sql).load::<NotifyPartial>(conn)
}

pub fn get_notify_by_data_id(
    conn: &PgConnection,
    data_id: &str,
) -> Result<Notify, diesel::result::Error> {
    use schema::notifies;

    notifies::table
        .filter(notifies::data_id.eq(data_id))
        .first::<Notify>(conn)
}

pub fn update_notify_status(
    conn: &PgConnection,
    data_id: &str,
    success: bool,
) -> Result<Notify, diesel::result::Error> {
    use schema::notifies;

    let notify = diesel::update(notifies::table.filter(notifies::data_id.eq(data_id)))
        .set((
            notifies::success.eq(success),
            notifies::retries.eq(notifies::retries + 1),
            notifies::updated_at.eq(Utc::now().naive_utc()),
        ))
        .get_result::<Notify>(conn);
    info!(
        "update notifies set success = {}, retries = retries + 1 where data_id = {}",
        success, data_id
    );

    notify
}

impl UserList {
    pub fn list(conn: &PgConnection, _topic: &str, offset: i64, limit: i64) -> Self {
        use schema::users::dsl::*;

        let result = users
            .filter(topic.eq(_topic))
            .order(updated_at.asc())
            .limit(limit)
            .offset(offset)
            .load::<User>(conn)
            .expect("loading users failed");

        let res = result
            .into_iter()
            .map(|mut v| {
                v.status = v.status.trim().to_string();
                v
            })
            .collect();
        UserList(res)
    }
}
