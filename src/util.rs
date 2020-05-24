use std::collections::HashMap;
use std::env;
extern crate chrono;
use anyhow::{anyhow, Result};

use dotenv::dotenv;

pub fn get_topics() -> Result<HashMap<String, String>> {
    dotenv().ok();
    let topic_str = env::var("TOPIC")?;
    let topics: Vec<&str> = topic_str.split(' ').collect();
    let mut res = HashMap::new();
    for item in topics {
        let pair: Vec<&str> = item.split(';').collect();
        if pair.len() != 2 {
            return Err(anyhow!(
                "got invalid TOPIC value from environment variable, TOPIC = {}",
                topic_str
            ));
        }
        res.insert(String::from(pair[0]), String::from(pair[1]));
    }
    Ok(res)
}

pub fn get_last_block_num_by_topic(topic: &str) -> String {
    return format!("{}_block_num", topic.to_lowercase());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_topics() {
        // multi topic
        let key = "TOPIC";
        let value = "b6b17424f87ffb8b5b853291f6dbaf0aac661ca2;https://xue-pub.prsdev.club/api/webhook/medium a7b751cc0e2f6c5be01ce95bc80b02d071022af4;https://box-pub.prsdev.club/api/webhook/medium";

        env::set_var(key, value);
        let topics = get_topics().unwrap();

        assert_eq!(topics.len() == 2, true);
        assert_eq!(
            topics.get("b6b17424f87ffb8b5b853291f6dbaf0aac661ca2"),
            Some(&"https://xue-pub.prsdev.club/api/webhook/medium".to_string())
        );
        assert_eq!(
            topics.get("a7b751cc0e2f6c5be01ce95bc80b02d071022af4"),
            Some(&"https://box-pub.prsdev.club/api/webhook/medium".to_string())
        );

        // single topic
        let value = "b6b17424f87ffb8b5b853291f6dbaf0aac661ca2;https://xue-pub.prsdev.club/api/webhook/medium";
        env::set_var(key, value);
        let topics = get_topics().unwrap();
        assert_eq!(topics.len() == 1, true);
        assert_eq!(
            topics.get("b6b17424f87ffb8b5b853291f6dbaf0aac661ca2"),
            Some(&"https://xue-pub.prsdev.club/api/webhook/medium".to_string())
        );
    }
}
