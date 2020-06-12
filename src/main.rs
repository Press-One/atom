use std::process;
extern crate curl;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

extern crate impl2001_rs;
extern crate prs_utility_rust;
extern crate qs_rs;
extern crate sentry;

use anyhow::Result;
use diesel::pg::PgConnection;
use std::env;
use std::thread;
use std::time::Duration;

mod crypto_util;
pub mod db;
mod frontmatter;
mod handlers;
mod processor;
mod prs;
mod settings;
mod url;
mod util;

use crate::impl2001_rs::pip::pip2001::Pip2001;
use crate::impl2001_rs::pip::Pip;

lazy_static! {
    static ref SETTINGS: settings::Settings = settings::Settings::load().unwrap();
}

fn main() {
    env_logger::init();
    init_sentry();

    let args: Vec<String> = env::args().collect();
    check_or_show_usage(&args);

    let command: &str = &args[1];
    info!("run command = {}", command);

    match command {
        "fetch" => run_fetch(),
        "syncserver" => run_syncserver(),
        "processpost" => process_post(),
        "atom" => generate_atom(),
        "web" => run_web(),
        _ => check_or_show_usage(&vec![]),
    }
}

fn init_sentry() {
    // init sentry
    let _guard;
    if let Some(sentry_dsn) = &SETTINGS.atom.sentry_dsn {
        _guard = sentry::init(String::from(sentry_dsn));
        sentry::integrations::panic::register_panic_handler();
    } else {
        debug!("can not get atom.sentry_dsn from toml config file, skip sentry integration");
    }
}

fn check_or_show_usage(args: &Vec<String>) {
    let usage = format!("usage: {} <fetch|syncserver|processpost|atom>", &args[0]);
    if args.len() <= 1 {
        println!("{}", usage);
        process::exit(0);
    }
}

fn run_fetch() {
    let db_conn_pool = db::establish_connection_pool();
    match db_conn_pool.get() {
        Ok(db_conn) => processor::fetchcontent(&db_conn),
        Err(e) => error!("db_conn_pool.get failed: {}", e),
    }
}

fn run_syncserver() {
    for item in &SETTINGS.topics {
        let topic = &item.topic;
        let _handle = thread::spawn(move || {
            let last_block_num_key = util::get_last_block_num_by_topic(&topic);
            let db_conn_pool = db::establish_connection_pool();
            loop {
                if let Ok(db_conn) = db_conn_pool.get() {
                    let start_block_num: i64;
                    if let Ok(last_block_num) = db::get_last_status(&db_conn, &last_block_num_key) {
                        start_block_num = last_block_num.val;
                    } else {
                        match prs::get_start_block_num_by_topic(&topic) {
                            Ok(v) => start_block_num = v as i64,
                            Err(e) => {
                                error!("get_start_block_num for topic: {} failed: {}", &topic, e);
                                continue;
                            }
                        }
                    }

                    if let Err(e) = sync_transactions(&db_conn, &topic, start_block_num) {
                        error!("sync_transactions for topic: {} failed: {}", &topic, e);
                    }
                    info!("sync transactions for topic: {} done. sleep...", &topic);
                } else {
                    error!("get database connection failed");
                }
                thread::sleep(Duration::from_millis(5000));
            }
        });
    }

    let handle_tx = thread::spawn(move || {
        let db_conn_pool = db::establish_connection_pool();

        loop {
            if let Ok(db_conn) = db_conn_pool.get() {
                synctxdata(&db_conn);
                processor::fetchcontent(&db_conn);

                if let Ok(unnotified_list) = db::get_unnotified_list(&db_conn) {
                    for item in &unnotified_list {
                        if let Err(e) = processor::check_and_send_webhook(&db_conn, &item.data_id) {
                            error!("check_and_send_webhook failed: {}", e);
                        }
                    }
                } else {
                    error!("get_unnotified_list failed");
                }
            } else {
                error!("get database connection failed");
            }
            thread::sleep(Duration::from_millis(10000));
        }
    });

    handle_tx.join().expect("handle_tx.join failed");
}

fn process_post() {
    let db_conn_pool = db::establish_connection_pool();
    if let Ok(db_conn) = db_conn_pool.get() {
        synctxdata(&db_conn);
        processor::fetchcontent(&db_conn);
    } else {
        error!("get database connection failed");
    }
}

fn generate_atom() {
    let db_conn_pool = db::establish_connection_pool();
    if let Ok(db_conn) = db_conn_pool.get() {
        if let Err(e) = processor::generate_atom_xml(&db_conn) {
            error!("generate_atom_xml failed: {}", e);
        }
    } else {
        error!("get database connection failed");
    }
}

fn run_web() {
    use actix_web::{middleware, web, App, HttpServer};

    let bind_address = &SETTINGS.atom.bind_address;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .data(db::establish_connection_pool())
            .service(web::resource("/users").route(web::get().to(handlers::users::list)))
            .service(
                web::resource("/json_posts").route(web::get().to(handlers::posts::list_all_asc)),
            )
            .service(
                web::resource("/posts").route(web::get().to(handlers::posts::list_all_atom_by_asc)),
            )
            .service(web::resource("/atom").route(web::get().to(handlers::posts::list_latest)))
    })
    .bind(&bind_address)
    .unwrap_or_else(|_| panic!("can not bind to {}", &bind_address))
    .run()
    .expect("HttpServer::new failed");
}

pub fn synctxdata(connection: &PgConnection) {
    let mut p: Pip2001 = Pip2001::new();
    let trxs_result = db::get_trxs(&connection, false);
    match trxs_result {
        Ok(trxs) => {
            for trx in trxs {
                let verify = match trx.verify_signature() {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "block_num = {}, trx verify_signature failed: {}",
                            trx.block_num, e
                        );
                        continue;
                    }
                };
                if verify {
                    debug!(
                        "block_num = {}, trx_id = {} verify success",
                        trx.block_num, trx.trx_id
                    );
                    let json_post_str = trx.to_post_json_str();
                    let post = p.from_json(&json_post_str);

                    match post {
                        Ok(Some(pipobject)) => {
                            let data: prs::Pip2001ActionData =
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
                            if let Err(e) = db::update_trx_status(connection, trx.block_num, true) {
                                error!(
                                    "update_trx_status failed: {}, block_num = {} processed = true",
                                    e, trx.block_num
                                );
                            }
                        }
                        Ok(None) => {
                            error!(
                                "Pip2001.from_json return None\ntrx = {:?}\njson_post_str = {}",
                                trx, json_post_str
                            );
                            continue;
                        }
                        Err(e) => {
                            error!(
                                "from_json failed: {:?}\ntrx = {:?}\njson_post_str = {}",
                                e, trx, json_post_str
                            );
                            continue;
                        }
                    }
                } else {
                    error!(
                        "block_num = {} trx_id = {}, verify failed",
                        trx.block_num, trx.trx_id
                    );
                    continue;
                }
            }
        }
        Err(e) => {
            error!("get_trxs failed: {:?}", e);
        }
    }
}

fn sync_transactions(conn: &PgConnection, topic: &str, start_block_num: i64) -> Result<()> {
    let mut start_block_num = start_block_num;
    let mut easy = prs::get_curl_easy().expect("get curl easy failed");
    loop {
        let transactions =
            prs::fetch_transactions_by_topic(&mut easy, &topic, start_block_num, 20)?;
        if transactions.is_empty() {
            return Ok(());
        }

        for trx in transactions {
            start_block_num = trx.block_num;
            debug!(
                "got block_num = {} topic = {}, new transaction: {:?}",
                trx.block_num, &topic, &trx
            );

            db::save_trx(&conn, &trx)?;
            let payload = match trx.get_notify_payload() {
                Ok(v) => match v {
                    Some(vv) => vv,
                    None => {
                        // PUBLISH_MANAGEMENT
                        continue;
                    }
                },
                Err(e) => {
                    error!("get_notify_payload failed: {}", e);
                    continue;
                }
            };

            let data_id = payload.block.data_id;
            db::save_notify(
                &conn,
                &data_id,
                payload.block.block_num,
                &payload.block.trx_id,
                &trx.get_topic(),
            )?;
            let key = util::get_last_block_num_by_topic(&trx.get_topic());
            db::update_last_status(&conn, &key, trx.block_num)?;
        }
    }
}
