table! {
    blocks (id) {
        id -> Int4,
        block_id -> Varchar,
        block_num -> Int8,
        block_type -> Varchar,
        block_timestamp -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

table! {
    contents (file_hash) {
        file_hash -> Bpchar,
        url -> Varchar,
        content -> Text,
        created_at -> Timestamp,
    }
}

table! {
    last_status (id) {
        id -> Int4,
        key -> Varchar,
        val -> Int8,
    }
}

table! {
    notifies (data_id) {
        data_id -> Varchar,
        block_num -> Int8,
        trx_id -> Varchar,
        success -> Bool,
        retries -> Int4,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        topic -> Varchar,
    }
}

table! {
    posts (id) {
        id -> Int4,
        publish_tx_id -> Bpchar,
        user_address -> Bpchar,
        file_hash -> Bpchar,
        topic -> Bpchar,
        url -> Varchar,
        update_by_tx_id -> Bpchar,
        updated_at -> Timestamp,
        fetched -> Bool,
        verify -> Bool,
        encryption -> Varchar,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        block_num -> Int8,
        data_type -> Varchar,
        data -> Varchar,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        trx_id -> Varchar,
        signature -> Varchar,
        hash -> Varchar,
        user_address -> Varchar,
    }
}

table! {
    users (user_address) {
        user_address -> Bpchar,
        status -> Bpchar,
        tx_id -> Bpchar,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    blocks,
    contents,
    last_status,
    notifies,
    posts,
    transactions,
    users,
);
