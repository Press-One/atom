extern crate impl2001_rs;

use chrono::prelude::Utc;
use diesel::pg::PgConnection;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{sink, Write};
use std::path::Path;

use dotenv::dotenv;

use crate::crypto_util;
use crate::impl2001_rs::pip::pip2001::Pip2001;
use crate::impl2001_rs::pip::pip2001::Pip2001MessageType;
use crate::impl2001_rs::pip::InputObject;
use crate::prs_utility_rust::utility;
use atom_syndication::{Feed, Generator, Person};

use crate::db;
use crate::db::models::{Post, PostPartial};
use crate::eos;
use crate::frontmatter;
use crate::util;

pub fn process_pip2001_message<'a>(
    conn: &PgConnection,
    pipobject: &Pip2001,
    tx_id: &'a str,
    user_pubaddr: &'a str,
    trx_table_num: i64,
    encryption: &str,
) -> bool {
    match pipobject.msg_type {
        Pip2001MessageType::PUBLISH_MANAGEMENT => {
            let mut users_action = "";
            let mut users_list = "";
            let mut topic = "";
            if pipobject.data.contains_key("allow") {
                users_action = "allow";
                users_list = &pipobject.data["allow"];
            } else if pipobject.data.contains_key("deny") {
                users_action = "deny";
                users_list = &pipobject.data["deny"];
            }
            if pipobject.data.contains_key("topic") {
                topic = &pipobject.data["topic"];
            } else {
                error!(
                    "can not find topic key from pipobject.data = {:?}",
                    &pipobject.data
                );
            }
            let now = Utc::now().naive_utc();
            for user_pubaddr in users_list.split(',') {
                debug!(
                    "tx_id = {} user = {} user_action = {:?}",
                    tx_id, user_pubaddr, users_action
                );
                db::save_user(&conn, &user_pubaddr, &users_action, &tx_id, &topic, now)
                    .expect("save user failed");
                db::update_last_status(&conn, "tx_num", trx_table_num)
                    .expect("update last_tx_num failed");
            }
        }
        Pip2001MessageType::PUBLISH => {
            let file_hash = &pipobject.data["file_hash"];
            let hash_alg = &pipobject.data["hash_alg"];
            let topic = &pipobject.data["topic"];
            let url: &str;
            let uris = &pipobject.meta["uris"];
            match uris {
                InputObject::String(_s) => {
                    error!(
                        "uris should be a url list, tx_id = {}\npipobject = {:?}",
                        tx_id, pipobject
                    );
                    return false;
                }
                InputObject::VecOfString(v) => {
                    url = &v[0];
                }
            }

            let now = Utc::now().naive_utc();
            // save updated_tx_id
            let mut updated_tx_id = "";
            if let Some(v) = pipobject.data.get("updated_tx_id") {
                updated_tx_id = v;
            }
            let _post = db::save_post(
                &conn,
                &tx_id,
                &user_pubaddr,
                &updated_tx_id,
                &file_hash,
                &hash_alg,
                &topic,
                &url,
                encryption,
                now,
            )
            .expect("save post failed");
            debug!(
                "post saved, file_hash = {} encryption = {}",
                _post.file_hash, _post.encryption
            );

            db::update_last_status(&conn, "tx_num", trx_table_num).expect(&format!(
                "update last_tx_num failed, tx_num = {}",
                trx_table_num
            ));
        }
        Pip2001MessageType::NA => warn!("Pip2001MessageType is NA"),
    }
    true
}

pub fn process_post_updated(connection: &PgConnection, post: &Post) -> bool {
    // 被更新的 publish_tx_id
    let updated_publish_tx_id = post.updated_tx_id.trim();

    if updated_publish_tx_id.len() == 0 {
        return true;
    }

    let updated_post_res = db::get_post_by_publish_tx_id(connection, &updated_publish_tx_id);
    match updated_post_res {
        Ok(updated_post) => {
            debug!(
                "process post updated, updated_publish_tx_id = {}",
                updated_publish_tx_id
            );

            // check user_address of updated post
            if updated_post.user_address != post.user_address {
                error!(
                    "update post failed, publish_tx_id: {}, updated user_address {} != post.user_address {}",
                    &post.publish_tx_id, updated_post.user_address, post.user_address
                );
                return false;
            } else {
                // delete old content
                debug!("delete content, file_hash = {}", updated_post.file_hash);
                if let Err(e) = db::delete_content(connection, &updated_post.file_hash) {
                    error!(
                        "delete content failed, file_hash = {}, error = {}",
                        updated_post.file_hash, e
                    );
                    return false;
                }

                // delete old post
                debug!("delete post, file_hash = {}", updated_post.file_hash);
                if let Err(e) = db::delete_post(connection, &updated_post.file_hash) {
                    error!(
                        "delete old content failed, file_hash = {}, error = {}",
                        updated_post.file_hash, e
                    );
                    return false;
                }
            }
        }
        Err(e) => {
            error!(
                "get_post_by_publish_tx_id failed, updated_publish_tx_id = {}, error: {}",
                updated_publish_tx_id, e
            );
            return false;
        }
    }

    true
}

pub fn fetchcontent(connection: &PgConnection) {
    let result_posts = db::get_posts(connection, false, 1000);
    match result_posts {
        Ok(posts) => {
            for post in posts {
                debug!("fetch file_hash = {} url = {}", post.file_hash, post.url);
                let response = fetch_markdown(post.url.clone());
                match response {
                    Ok(data) => {
                        let html;
                        if !post.encryption.is_empty() {
                            let enc_post: eos::EncPost = serde_json::from_slice(&data.as_bytes())
                                .expect("parse encryption post failed");
                            let dec_html_result =
                                decrypt_aes_256_cbc(&enc_post.session, &enc_post.content);
                            match dec_html_result {
                                Ok(dec_html) => html = dec_html,
                                Err(e) => {
                                    error!(
                                        "decrypt enc post file_hash = {} failed: {:?}",
                                        post.file_hash, e
                                    );
                                    continue;
                                }
                            }
                        } else {
                            html = data;
                        }
                        let hex = utility::hash_text(&html, &post.hash_alg)
                            .ok()
                            .expect(&format!(
                                "utility::hash_text failed, hash_alg = {} html = {}",
                                &post.hash_alg, &html
                            ));
                        // just check and output error message
                        if hex != post.file_hash {
                            error!(
                                "hex != file_hash, hash_alg = {} hex = {} file_hash = {} url = {}",
                                &post.hash_alg, hex, post.file_hash, post.url
                            );
                        }

                        let content = db::get_content(connection, &post.file_hash);
                        match content {
                            Ok(_) => {
                                debug!("content already exists, file_hash = {}", &post.file_hash);
                            }
                            Err(e) => {
                                if e == diesel::NotFound {
                                    if let Err(e) = db::save_content(
                                        connection,
                                        &post.file_hash,
                                        &post.url,
                                        &html,
                                    ) {
                                        error!(
                                            "save_content file_hash = {} url = {} failed: {:?}",
                                            &post.file_hash, &post.url, e
                                        );
                                        continue;
                                    }
                                } else {
                                    error!("get_content failed: {}", e);
                                }
                            }
                        }

                        db::update_post_status(connection, &post.file_hash, true, true)
                            .expect("update_post_status failed");

                        if !process_post_updated(connection, &post) {
                            error!(
                                "post/content update failed, post.file_hash = {}, skip",
                                post.file_hash
                            );
                            continue;
                        }
                    }
                    Err(e) => {
                        error!("fetch_markdown {} failed: {:?}", &post.url, e);
                        continue;
                    }
                }
            }
        }
        Err(e) => error!("get posts failed: {:?}", e),
    }
}

pub fn fetch_markdown(url: String) -> std::result::Result<String, Box<dyn Error>> {
    let mut easy = eos::get_curl_easy()?;
    easy.url(&url)?;
    let _redirect = easy.follow_location(true);
    let mut data = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    };

    let html = String::from_utf8(data).expect("body is not valid UTF8!");
    let result = easy.response_code();
    match result {
        Ok(respcode) => {
            if respcode == 200 {
                Ok(html)
            } else {
                Err(From::from(format!(
                    "url = {} error status code: {:?}",
                    url, respcode,
                )))
            }
        }
        Err(e) => Err(From::from(format!("url = {} error = {}", url, e))),
    }
}

fn decrypt_aes_256_cbc(session: &str, content: &str) -> Result<String, String> {
    dotenv().ok();
    let encryption_key = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let iv_prefix = env::var("IV_PREFIX").expect("IV_PREFIX must be set");
    let hashiv = crypto_util::get_iv(&iv_prefix, session);
    let key = hex::decode(&encryption_key).expect(&format!(
        "hex::decode failed, encryption_key = {}",
        encryption_key
    ));

    crypto_util::decrypt_aes_256_cbc(String::from(content), &key, hashiv)
}

pub fn generate_atom_xml(connection: &PgConnection) {
    dotenv().ok();
    let xml_output_dir = env::var("XML_OUTPUT_DIR").expect("XML_OUTPUT_DIR must be set");
    fs::create_dir_all(&xml_output_dir).expect("create xml_output_dir failed");

    let topics_map = util::get_topics();
    for item in topics_map {
        let topic = item.0;
        debug!("generate atom for topic = {}", topic);
        let posts_result = db::get_allow_posts(&connection, &topic);
        match posts_result {
            Ok(posts) => {
                let atomstring = atom(&connection, posts);

                let fpath = Path::new(&xml_output_dir).join(&topic);
                let mut file = match fs::File::create(&fpath) {
                    Ok(file) => file,
                    Err(e) => panic!(
                        "create file failed: {}, fpath = {}",
                        fpath.as_os_str().to_string_lossy(),
                        e
                    ),
                };
                file.write_all(atomstring.as_bytes())
                    .expect("write all failed");
            }
            Err(e) => error!("get_allow_posts failed: {}", e),
        }
    }
}

pub fn atom(connection: &PgConnection, posts: Vec<PostPartial>) -> String {
    use atom_syndication::Content;
    use atom_syndication::Entry;

    let mut generator = Generator::default();
    generator.set_value("PRESSone Atom Generator");

    let mut feed = Feed::default();
    feed.set_generator(generator);
    let mut entries = Vec::new();

    for post in posts {
        debug!("generate atom for post file_hash = {} ", post.file_hash);
        let result_content = db::get_content(connection, &post.file_hash);
        match result_content {
            Ok(post_content) => {
                let markdown_attrs = frontmatter::parse(&post_content.content);
                debug!(
                    "post content title = {} author = {} published = {}",
                    markdown_attrs.title, markdown_attrs.author, markdown_attrs.published
                );
                let mut feed_content = Content::default();
                feed_content.set_content_type("text/markdown".to_string());
                feed_content.set_value(format!("<![CDATA[{}]]>", post_content.content));

                let mut person = Person::default();
                person.set_name(&markdown_attrs.author);
                let mut entry = Entry::default();

                entry.set_id(&post.publish_tx_id);
                entry.set_title(&markdown_attrs.title);
                entry.set_published(markdown_attrs.published);
                entry.set_authors(vec![person]);
                entry.set_content(feed_content);
                entries.push(entry);
                // check and send webhook notify
                check_and_send_webhook(connection, &post.publish_tx_id);
            }
            Err(e) => error!("get content failed: {:?}", e),
        }
    }

    let mut feed = Feed::default();
    feed.set_entries(entries);

    feed.write_to(sink()).expect("feed.write_to failed");
    feed.to_string()
}

pub fn check_and_send_webhook(conn: &PgConnection, data_id: &str) {
    let notify_result = db::get_notify_by_data_id(conn, data_id);
    match notify_result {
        Ok(notify) => {
            debug!(
                "notify data_id = {} topic = {} success = {}",
                notify.data_id, notify.topic, notify.success
            );
            if notify.success {
                debug!(
                    "block_num = {} trx_id = {} notify webhook success already, skip ...",
                    notify.block_num, notify.trx_id
                );
                return;
            }
            let payload = eos::NotifyPayload {
                block: eos::NotifyBlock {
                    data_id: notify.data_id.clone(),
                    block_num: notify.block_num,
                    trx_id: notify.trx_id,
                },
            };
            let topics_map = util::get_topics();
            if let Some(notify_url) = topics_map.get(&notify.topic) {
                debug!("send notify payload to {}", notify_url);
                match eos::notify_webhook(&payload, notify_url) {
                    Ok(status_code) => {
                        let success = status_code == 200;
                        db::update_notify_status(conn, &notify.data_id, success)
                            .expect("update_notify_status failed");
                    }
                    Err(e) => error!(
                        "block_num = {}, url = {}, notify_webhook failed: {}",
                        notify.block_num, notify_url, e
                    ),
                }
            } else {
                error!("can not find webhook url for topic = {}", notify.topic);
            }
        }
        Err(e) => error!("get notify by data_id = {} failed: {:?}", data_id, e),
    }
}
