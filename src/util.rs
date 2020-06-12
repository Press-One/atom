pub fn get_last_block_num_by_topic(topic: &str) -> String {
    return format!("{}_block_num", topic.to_lowercase());
}
