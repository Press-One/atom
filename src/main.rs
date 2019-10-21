use std::process;
extern crate crypto;
extern crate curl;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
extern crate impl2001_rs;
extern crate prs_utility_rust;
extern crate qs_rs;
extern crate sentry;

use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use std::thread;
use std::time::Duration;

mod crypto_util;
pub mod db;
mod eos;
mod frontmatter;
mod processor;
mod util;

use crate::impl2001_rs::pip::pip2001::Pip2001;
use crate::impl2001_rs::pip::Pip;

fn main() {
    env_logger::init();
    // init sentry
    let _guard;
    if let Ok(sentry_dsn) = env::var("SENTRY_DSN") {
        _guard = sentry::init(sentry_dsn);
        sentry::integrations::panic::register_panic_handler();
    } else {
        debug!("can not get SENTRY_DSN environment variable, skip sentry integration");
    }

    let args: Vec<String> = env::args().collect();
    let command: &str;

    let usage = format!("usage: {} <fetch|syncserver|atom|web>", &args[0]);
    if args.len() <= 1 {
        println!("{}", usage);
        process::exit(0);
    }

    command = &args[1];
    info!("command = {}", command);

    match command {
        "fetch" => {
            let db_conn_pool = db::establish_connection_pool();
            if let Ok(db_conn) = db_conn_pool.get() {
                processor::fetchcontent(&db_conn);
            }
        }
        "syncserver" => {
            let _handle = thread::spawn(move || {
                let db_conn_pool = db::establish_connection_pool();
                loop {
                    if let Ok(db_conn) = db_conn_pool.get() {
                        let start_block_num: i64;
                        if args.len() == 3 {
                            start_block_num = args[2]
                                .parse()
                                .expect("parse start_block_num from command line args failed");
                        } else if let Ok(last_block_num) =
                            db::get_last_status(&db_conn, "block_num")
                        {
                            start_block_num = last_block_num.val;
                        } else {
                            error!("get last_block_num failed");
                            thread::sleep(Duration::from_millis(1000));
                            continue;
                        }

                        if let Ok(mut easy) = eos::get_curl_easy() {
                            let info_result = eos::get_info(&mut easy);
                            match info_result {
                                Ok(info) => {
                                    let last_irreversible_block_num =
                                        info.data.last_irreversible_block_num;
                                    info!("last block number {:?}", last_irreversible_block_num);
                                    if last_irreversible_block_num > start_block_num {
                                        debug!("find new blocks!");
                                        sync_blocks(&db_conn, start_block_num);
                                    }
                                }
                                Err(e) => error!("get info failed: {:?}", e),
                            }
                            info!("sync blocks done. sleep...");
                        } else {
                            error!("get curl easy failed");
                        }
                    } else {
                        error!("get database connection failed");
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            });

            let handle_tx = thread::spawn(move || {
                let db_conn_pool = db::establish_connection_pool();

                loop {
                    if let Ok(db_conn) = db_conn_pool.get() {
                        let last_tx_num: i64;
                        let saved_tx_num = db::get_last_status(&db_conn, "tx_num");
                        match saved_tx_num {
                            Ok(v) => last_tx_num = v.val,
                            Err(e) => {
                                if e == diesel::NotFound {
                                    last_tx_num = 0;
                                } else {
                                    error!("get last tx_num failed: {}", e);
                                    continue;
                                }
                            }
                        }
                        if let Ok(max_tx_num) = db::get_max_tx_num(&db_conn) {
                            if i64::from(max_tx_num) > last_tx_num {
                                debug!(
                                    "get new tx, last tx number {:?} max tx num {:?}",
                                    last_tx_num, max_tx_num
                                );
                                synctxdata(&db_conn);
                                processor::fetchcontent(&db_conn);
                                processor::generate_atom_xml(&db_conn);
                            }
                        } else {
                            error!("get max_tx_num failed");
                        }
                    } else {
                        error!("get database connection failed");
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            });

            // _handle.join().unwrap();
            handle_tx.join().unwrap();
        }
        "atom" => {
            let db_conn_pool = db::establish_connection_pool();
            if let Ok(db_conn) = db_conn_pool.get() {
                processor::generate_atom_xml(&db_conn);
            } else {
                error!("get database connection failed");
            }
        }
        "web" => {
            use actix_cors::Cors;
            use actix_web::{web, App, HttpRequest, HttpServer, Responder};

            dotenv().ok();
            let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be set");
            fn output_xml(req: HttpRequest) -> impl Responder {
                let topic = req.match_info().get("topic").expect("can not get topic");
                let db_conn_pool = db::establish_connection_pool();
                if let Ok(db_conn) = db_conn_pool.get() {
                    processor::atom(&db_conn, &topic)
                } else {
                    let msg = "get database connection failed";
                    error!("{}", msg);
                    String::from(msg)
                }
            }

            HttpServer::new(|| {
                App::new()
                    .wrap(
                        Cors::new()
                            .allowed_origin("http://localhost:4008")
                            .allowed_origin("https://box-posts.press.one")
                            .allowed_origin("https://xue-posts.press.one")
                            .allowed_origin("https://box-posts.xue.cn")
                            .allowed_origin("https://xue-posts.xue.cn"),
                    )
                    .route("/output/{topic}", web::get().to(output_xml))
            })
            .bind(&bind_address)
            .unwrap_or_else(|_| panic!("can not bind to {}", &bind_address))
            .run()
            .unwrap();
        }
        _ => {
            println!("{}", usage);
            process::exit(0);
        }
    }
}

pub fn synctxdata(connection: &PgConnection) {
    let mut p: Pip2001 = Pip2001::new();
    let trxs_result = db::get_confirmed_trxs(&connection);
    match trxs_result {
        Ok(trxs) => {
            for trx in trxs {
                let verify = trx.verify_signature();
                if verify {
                    let json_post_str = trx.to_post_json_str();
                    let post = p.from_json(&json_post_str);

                    match post {
                        Ok(Some(pipobject)) => {
                            let data: eos::Pip2001ActionData =
                                serde_json::from_str(&trx.data).expect("parse trx data failed");
                            // verify user pubaddr and sign
                            let encryption = data.get_encryption();
                            let _result = processor::process_pip2001_message(
                                connection,
                                &pipobject,
                                &data.id,
                                &trx.user_address,
                                i64::from(trx.id),
                                &encryption,
                            );
                        }
                        Ok(None) => {
                            panic!("Pip2001.from_json return None");
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                } else {
                    error!(
                        "block_num = {} trx_id = {}, verify failed",
                        trx.block_num, trx.trx_id
                    );
                }
            }
        }
        Err(e) => {
            error!("get_confirmed_trxs failed: {:?}", e);
        }
    }
}

fn sync_blocks(conn: &PgConnection, start_block_num: i64) {
    let env_thread_num = env::var("THREAD_NUM").expect("THREAD_NUM must be set");
    let thread_num: u32 = env_thread_num.parse::<u32>().unwrap();
    let topics_map = util::get_topics();
    let topics: Vec<String> = topics_map.iter().map(|(topic, _)| topic.clone()).collect();
    let blocksbatch = eos::BlockIteratorBatch::new(thread_num, start_block_num);
    for blocks in blocksbatch {
        for block in blocks {
            let should_update_block_num = block.block_num % 100 == 0;
            let block_type = db::get_block_type(&block);
            if block_type == db::models::BlockType::EMPTY {
                debug!(
                    "block_num = {}, empty transaction data, skip ...",
                    block.block_num
                );
                if should_update_block_num {
                    if let Err(e) = db::update_last_status(&conn, "block_num", block.block_num) {
                        error!("update last_block_num failed: {:?}", e);
                    }
                }
            } else {
                if block_type == db::models::BlockType::DATA && !block.has_topic(&topics) {
                    // confirm block 没有 topic 字段
                    info!(
                        "block_num = {} not contains by topics = {:?}, skip ...",
                        block.block_num, topics
                    );
                    if should_update_block_num {
                        if let Err(e) = db::update_last_status(&conn, "block_num", block.block_num)
                        {
                            error!("update last_block_num failed: {:?}", e);
                        }
                    }
                    continue;
                }

                debug!("new block: {:?}", block);
                let result = db::save_block(&conn, &block);
                match result {
                    Ok(_) => {
                        if let Err(e) = db::update_last_status(&conn, "block_num", block.block_num)
                        {
                            error!("update last_block_num failed: {:?}", e);
                        }
                    }
                    Err(e) => error!("save block failed: {}", e),
                }
                for payload in block.get_notify_payloads() {
                    let mut notify_topic = String::from("");
                    let data_id = payload.block.data_id;
                    if let Some(_v) = block.get_topic_by_data_id(&data_id) {
                        notify_topic = _v;
                    }
                    db::save_notify(
                        &conn,
                        &data_id,
                        payload.block.block_num,
                        &payload.block.trx_id,
                        &notify_topic,
                    )
                    .expect("save notify failed");
                }
            }
        }
    }
}
