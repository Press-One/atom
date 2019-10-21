extern crate chrono;
extern crate dotenv;

use chrono::prelude::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use dotenv::dotenv;
use std::env;
use std::time::Duration;

pub mod models;
pub mod schema;
use super::eos;

use self::models::{Block, BlockType, NewBlock, TrxStatus};
use self::models::{Content, NewContent};
use self::models::{LastStatus, NewLastStatus};
use self::models::{NewNotify, Notify, NotifyPartial};
use self::models::{NewPost, Post, PostPartial};
use self::models::{NewTrx, Trx};
use self::models::{NewUser, User};

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
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("create database pool failed")
}

pub fn save_user<'a>(
    conn: &PgConnection,
    user_address: &'a str,
    status: &'a str,
    tx_id: &'a str,
    updated_at: chrono::NaiveDateTime,
) -> Result<User, diesel::result::Error> {
    use schema::users;

    let new_user = NewUser {
        user_address,
        status,
        tx_id,
        updated_at,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .on_conflict(users::user_address)
        .do_update()
        .set(&new_user)
        .get_result(conn)
    //.expect("Error update users") //TODO: stop? log? set status to warning?
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
pub fn save_post<'a>(
    conn: &PgConnection,
    publish_tx_id: &'a str,
    user_address: &'a str,
    update_by_tx_id: &'a str,
    file_hash: &'a str,
    topic: &'a str,
    url: &'a str,
    encryption: &str,
    updated_at: chrono::NaiveDateTime,
) -> Result<Post, diesel::result::Error> {
    use schema::posts;

    let new_post = NewPost {
        publish_tx_id,
        user_address,
        update_by_tx_id,
        file_hash,
        topic,
        url,
        encryption,
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

pub fn get_posts(
    conn: &PgConnection,
    fetch_status: bool,
    limit: i64,
) -> Result<Vec<Post>, diesel::result::Error> {
    use schema::posts::dsl::*;
    posts
        .filter(fetched.eq(fetch_status))
        .limit(limit)
        .load::<Post>(conn)
}

pub fn get_all_posts(conn: &PgConnection) -> Result<Vec<Post>, diesel::result::Error> {
    use schema::posts::dsl::*;

    posts.load::<Post>(conn)
}

pub fn get_allow_posts(
    conn: &PgConnection,
    topic: &str,
) -> Result<Vec<PostPartial>, diesel::result::Error> {
    let sql = format!(
        r#"
        SELECT posts.publish_tx_id, posts.file_hash, posts.topic
        FROM posts, users
        WHERE posts.user_address = users.user_address
        AND topic = '{}'
        AND posts.fetched = 't'
        AND posts.verify = 't'
        AND users.status = 'allow'"#,
        topic
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

    let result = transactions
        .filter(status.eq(models::TrxStatus::CONFIRMED.to_string()))
        .order(id.desc())
        .first::<Trx>(conn);
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

pub fn save_trx(
    conn: &PgConnection,
    block_num: i64,
    trx_id: &str,
    data_type: &str,
    data: &str,
) -> Result<Trx, diesel::result::Error> {
    use schema::transactions;

    let action_data: eos::Pip2001ActionData =
        serde_json::from_str(data).expect("parse trx data failed");
    let new_trx = NewTrx {
        block_num,
        data_type: &action_data._type,
        data,
        status: &TrxStatus::SUBMITTED.to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: None,
        trx_id,
        signature: &action_data.signature,
        hash: &action_data.hash,
        user_address: &action_data.user_address,
    };

    let trx = diesel::insert_into(transactions::table)
        .values(&new_trx)
        .on_conflict(transactions::trx_id)
        .do_update()
        .set(&new_trx)
        .get_result(conn);

    info!(
        "saved trx data from block_num = {}, data_type = {}",
        block_num, data_type
    );

    trx
}

pub fn get_confirmed_trxs(conn: &PgConnection) -> Result<Vec<Trx>, diesel::result::Error> {
    use schema::transactions::dsl::*;

    transactions
        .filter(status.eq(models::TrxStatus::CONFIRMED.to_string()))
        .load::<Trx>(conn)
}

pub fn save_block(
    conn: &PgConnection,
    eos_block: &eos::Block,
) -> Result<Block, diesel::result::Error> {
    use schema::blocks;

    let block_num: i64 = eos_block.block_num;
    let block_type = &get_block_type(&eos_block).to_string();

    let new_block = NewBlock {
        block_id: &eos_block.block_id,
        block_num,
        block_type,
        block_timestamp: &eos_block.timestamp,
        created_at: Utc::now().naive_utc(),
        updated_at: None,
    };
    let block = diesel::insert_into(blocks::table)
        .values(&new_block)
        .on_conflict(blocks::block_num)
        .do_update()
        .set(&new_block)
        .get_result(conn);

    info!(
        "saved block_num = {}, block_type = {}",
        eos_block.block_num, block_type
    );

    for trx in &eos_block.trxs {
        for action in &trx.actions {
            match &action {
                eos::Pip2001Action::Data(data) => {
                    let data = data.clone();
                    let data_type = &data._type;
                    let data_str = serde_json::to_string(&data).expect("can not dumps action data");
                    let trx_id = &trx.trx_id;
                    save_trx(conn, block_num, trx_id, data_type, &data_str)?;
                }
                eos::Pip2001Action::Validation(data) => {
                    let inner_data: eos::Pip2001ActionValidationInnerData =
                        serde_json::from_str(&data.data)
                            .expect("can not loads validation action data");
                    // FIXME: hardcode
                    if inner_data.result == "VALID" && data.oracleservice == "prs.oracle" {
                        let status = TrxStatus::CONFIRMED.to_string();
                        update_trx_status(conn, &inner_data.trx_id, &status)?;
                    }
                }
            }
        }
    }

    block
}

pub fn update_trx_status(
    conn: &PgConnection,
    ref_trx_id: &str,
    status: &str,
) -> Result<Trx, diesel::result::Error> {
    use schema::transactions;

    let trx = diesel::update(transactions::table.filter(transactions::trx_id.eq(ref_trx_id)))
        .set((
            transactions::status.eq(status),
            transactions::updated_at.eq(Utc::now().naive_utc()),
        ))
        .get_result::<Trx>(conn);
    info!(
        "update transactions set status = {} where trx_id = {}",
        status, ref_trx_id
    );

    trx
}

pub fn get_block(conn: &PgConnection, block_id: &str) -> Result<Block, diesel::result::Error> {
    use schema::blocks;

    blocks::table
        .filter(blocks::block_id.eq(block_id))
        .first::<Block>(conn)
}

pub fn get_block_type(block: &eos::Block) -> BlockType {
    // FIXME: should check action_type not block_type
    for transaction in &block.trxs {
        for action in &transaction.actions {
            match action {
                eos::Pip2001Action::Data(data) => {
                    if data._type == "PIP:2001" {
                        // FIXME: hardcode
                        return BlockType::DATA;
                    }
                }
                eos::Pip2001Action::Validation(data) => {
                    if data._type == "PIP:2001" {
                        // FIXME: hardcode
                        return BlockType::CONFIRM;
                    }
                }
            }
        }
    }

    BlockType::EMPTY
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
        .on_conflict(notifies::trx_id)
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
