extern crate jobpool;

use curl::easy::{Easy, List};
use dotenv::dotenv;
use jobpool::JobPool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::Read;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainInfo {
    pub errors: Option<String>,
    pub data: ChainInfoData,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainInfoData {
    pub server_version: String,
    pub chain_id: String,
    pub head_block_num: i64,
    pub last_irreversible_block_num: i64,
    pub last_irreversible_block_id: String,
    pub head_block_id: String,
    head_block_time: String,
    head_block_producer: String,
    virtual_block_cpu_limit: String,
    virtual_block_net_limit: u64,
    block_cpu_limit: u64,
    block_net_limit: u64,
    server_version_string: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pip2001ActionValidation {
    // pub user: String,
    pub oracleservice: String,
    pub auth_hash: i64,
    #[serde(rename = "type")]
    pub _type: String,
    pub meta: String,
    pub data: String,
    pub memo: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Pip2001Action {
    Data(Pip2001ActionData),
    Validation(Pip2001ActionValidation),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pip2001Trx {
    pub trx_id: String,
    pub actions: Vec<Pip2001Action>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub block_num: i64,
    pub block_id: String,
    pub trxs: Vec<Pip2001Trx>,
    pub timestamp: String,
}

impl Block {
    pub fn get_topics(&self) -> HashMap<String, Option<String>> {
        let mut res: HashMap<String, Option<String>> = HashMap::new();
        for transaction in &self.trxs {
            for action in &transaction.actions {
                if let Pip2001Action::Data(data) = &action {
                    let data_id = String::from(&data.id);
                    let inner_data: Value =
                        serde_json::from_str(&data.data).expect("parse inner data failed");
                    if inner_data["topic"].is_null() {
                        continue;
                    }
                    if let Value::String(_v) = &inner_data["topic"] {
                        res.insert(data_id, Some(String::from(_v)));
                    } else {
                        res.insert(data_id, None);
                    }
                }
            }
        }
        res
    }

    pub fn get_topic_by_data_id(&self, data_id: &str) -> Option<String> {
        let block_topics = self.get_topics();
        for (_data_id, topic) in block_topics.iter() {
            if _data_id == data_id {
                return topic.clone();
            }
        }
        None
    }

    pub fn has_topic(&self, env_topics: &[String]) -> bool {
        let block_topics = self.get_topics();
        for (_, topic) in block_topics.iter() {
            if let Some(item) = topic {
                if env_topics.contains(item) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_notify_payloads(&self) -> Vec<NotifyPayload> {
        let mut v = Vec::new();
        for transaction in &self.trxs {
            for action in &transaction.actions {
                match &action {
                    Pip2001Action::Data(data) => {
                        if self.get_topic_by_data_id(&data.id).is_some() {
                            let payload = NotifyPayload {
                                block: NotifyBlock {
                                    data_id: data.id.clone(),
                                    block_num: self.block_num,
                                    trx_id: transaction.trx_id.clone(),
                                },
                            };
                            v.push(payload);
                        }
                    }
                    Pip2001Action::Validation(_) => {
                        continue;
                    }
                }
            }
        }
        v
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pip2001ActionData {
    pub id: String,
    pub user_address: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub meta: String, // json.dumps string
    pub data: String, // json.dumps string
    pub hash: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pip2001ActionValidationInnerData {
    pub trx_id: String,
    pub result: String,
}

impl Pip2001ActionData {
    pub fn get_encryption(&self) -> String {
        let v: Value = serde_json::from_str(&self.meta).expect("parse meta failed");
        if !v["encryption"].is_null() {
            if let Value::String(_v) = &v["encryption"] {
                return _v.clone();
            }
        }
        String::from("")
    }
}

pub struct BlockIteratorBatch {
    next_block_num: i64,
    thread_num: u32,
    pool: JobPool,
}

impl BlockIteratorBatch {
    pub fn new(thread_num: u32, block_num: i64) -> BlockIteratorBatch {
        info!("start block_num = {}", block_num);
        let pool = JobPool::new(thread_num as usize);
        BlockIteratorBatch {
            next_block_num: block_num,
            thread_num,
            pool,
        }
    }
}

impl Iterator for BlockIteratorBatch {
    type Item = Vec<Block>;
    fn next(&mut self) -> Option<Self::Item> {
        let (tx, rx) = mpsc::channel();
        for i in 0..self.thread_num {
            let tx = tx.clone();

            // TODO: use the max block number
            let block_num = self.next_block_num + i64::from(i);
            self.pool.queue(move || {
                // Do some work, following is just for example's sake
                let mut easy = get_curl_easy().expect("get curl easy failed");
                let result = get_block(&mut easy, block_num);
                debug!(
                    "BlockIteratorBatch block_num = {} in thread {}",
                    block_num, i
                );
                match result {
                    Ok(eos_block) => {
                        tx.send(Some(eos_block)).unwrap();
                    }
                    Err(e) => {
                        error!("get_block failed: {}", e);
                        tx.send(None).unwrap();
                    }
                }
            });
        }

        let mut all_blocks = Vec::new();
        for _ in 0..self.thread_num {
            match rx.recv() {
                Ok(val) => {
                    if let Some(v) = val {
                        all_blocks.push(v);
                    }
                }
                Err(e) => error!("rx recv error: {}", e),
            }
        }

        all_blocks.sort_by(|a, b| a.block_num.cmp(&b.block_num));
        let mut max_block_num = self.next_block_num as i64;
        for block in &all_blocks {
            if max_block_num == block.block_num {
                max_block_num = block.block_num + 1;
            }
        }
        self.next_block_num = max_block_num;

        println!("max block num  {}", max_block_num);

        Some(all_blocks)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyPayload {
    pub block: NotifyBlock,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyBlock {
    #[serde(rename = "id")]
    pub data_id: String,
    #[serde(rename = "blockNum")]
    pub block_num: i64,
    #[serde(rename = "blockTransactionId")]
    pub trx_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncPost {
    pub session: String,
    pub content: String,
}

fn get_base_url() -> String {
    dotenv().ok();
    env::var("EOS_BASE_URL").expect("get eos base url")
}

fn get_url(suffix: &str) -> String {
    let base_url = get_base_url();
    format!("{}{}", base_url, String::from(suffix))
}

pub fn get_curl_easy() -> Result<Easy, Box<dyn Error>> {
    // keep alive
    let mut easy = Easy::new();
    easy.connect_timeout(Duration::from_secs(10))?;
    easy.timeout(Duration::from_secs(10))?;

    Ok(easy)
}

#[allow(dead_code)]
pub fn get_info(easy: &mut Easy) -> Result<ChainInfo, Box<dyn Error>> {
    let url = get_base_url();
    debug!("get chain url = {}", url);
    easy.url(&url)?;
    let mut response_content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    };
    let info = serde_json::from_slice(&response_content)?;
    Ok(info)
}

pub fn get_block(easy: &mut Easy, block_num: i64) -> Result<Block, Box<dyn Error>> {
    let url_suffix = format!("/blocks/{}", block_num);
    let mut block_id: String = String::from("");
    let mut timestamp: String = String::from("");
    let mut trxs: Vec<Pip2001Trx> = Vec::new();

    let url = get_url(&url_suffix);
    debug!("get block url = {}", url);
    easy.url(&url)?;
    let mut response_content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_content.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    };

    let res: Value = serde_json::from_slice(&response_content)?;

    // check if response success
    if let Value::Bool(success) = &res["success"] {
        if !success {
            let response_text = String::from_utf8_lossy(&response_content);
            // FIXME: hardcode
            let err_not_find_block_pattern = "Could not find block: ";
            if response_text.contains(err_not_find_block_pattern) {
                let err_msg = format!("last irreversible block_num = {} not generated", block_num);
                return Err(From::from(err_msg));
            } else {
                return Err(From::from(response_text));
            }
        }
    }

    if let Value::Object(data) = &res["data"] {
        if let Value::Number(_v) = &data["block_num"] {
            let _vv = _v.as_i64();
            if let Some(_vvv) = _vv {
                let _block_num = _vvv;
                if _block_num != block_num {
                    return Err(From::from(format!(
                        "block_num error, {} != {}",
                        block_num, _block_num
                    )));
                }
            } else {
                return Err(From::from(format!(
                    "block_num = {}, can not find block_num from response",
                    block_num
                )));
            }
        }
        if let Value::String(_v) = &data["id"] {
            block_id = _v.clone();
        } else {
            return Err(From::from(format!(
                "block_num = {}, can not find block_id",
                block_num
            )));
        }

        if let Value::String(_v) = &data["timestamp"] {
            timestamp = _v.clone();
        } else {
            return Err(From::from(format!(
                "block_num = {}, can not find block timestamp",
                block_num
            )));
        }

        if let Value::Array(transactions) = &data["transactions"] {
            for trx in transactions {
                let trx_id: String;
                let mut actions: Vec<Pip2001Action> = Vec::new();
                if trx["trx"].is_null() {
                    error!("block_num = {}, can not transactions.trx", block_num);
                    continue;
                }

                if let Value::String(_v) = &trx["trx"]["id"] {
                    trx_id = _v.clone();
                } else {
                    error!("block_num = {}, can not trx.id", block_num);
                    continue;
                }

                if trx["trx"]["transaction"].is_null() {
                    error!("block_num = {}, can not trx.transaction", block_num);
                    continue;
                }

                if let Value::Array(trx_actions) = &trx["trx"]["transaction"]["actions"] {
                    for trx_action in trx_actions {
                        if trx_action["data"].is_null() {
                            error!("block_num = {}, can not find action.data", block_num);
                            continue;
                        }
                        let data_res: Result<Pip2001Action, serde_json::error::Error> =
                            serde_json::from_value(trx_action["data"].to_owned());
                        match data_res {
                            Ok(data) => actions.push(data),
                            Err(e) => info!(
                                "block_num = {}, unsupport action data, error = {:?}",
                                block_num, e
                            ),
                        }
                    }
                }
                trxs.push(Pip2001Trx { trx_id, actions });
            }
        }
    }

    Ok(Block {
        block_num,
        block_id,
        trxs,
        timestamp,
    })
}

pub fn notify_webhook(payload: &NotifyPayload, url: &str) -> (u32, String) {
    debug!("notify webhook url = {}", url);
    let mut easy = get_curl_easy().expect("get curl easy failed");
    easy.url(&url).unwrap();
    let mut headers = List::new();
    headers.append("Content-Type: application/json").unwrap();
    easy.http_headers(headers).unwrap();
    easy.post(true).unwrap();
    let payload = serde_json::to_string(&payload).unwrap();
    debug!(
        "curl -X POST -H 'Content-Type: application/json' -d '{}' {}",
        payload, url
    );
    let mut payload_bytes = payload.as_bytes();
    easy.post_field_size(payload_bytes.len() as u64).unwrap();
    let mut response_content = Vec::new();

    {
        let mut transfer = easy.transfer();
        transfer
            .read_function(|buf| Ok(payload_bytes.read(buf).unwrap_or(0)))
            .unwrap();
        transfer
            .write_function(|data| {
                response_content.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    let status_code = easy.response_code().unwrap();
    let msg = String::from_utf8_lossy(&response_content);
    (status_code, msg.to_string())
}
