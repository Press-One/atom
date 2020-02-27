table! {
    contents (file_hash) {
        file_hash -> Bpchar,
        url -> Varchar,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted -> Bool,
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
        updated_tx_id -> Bpchar,
        updated_at -> Timestamp,
        fetched -> Bool,
        verify -> Bool,
        encryption -> Varchar,
        hash_alg -> Varchar,
        deleted -> Bool,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        block_num -> Int8,
        data_type -> Varchar,
        data -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        trx_id -> Varchar,
        signature -> Varchar,
        hash -> Varchar,
        user_address -> Varchar,
        processed -> Bool,
    }
}

table! {
    users (user_address) {
        user_address -> Bpchar,
        status -> Bpchar,
        tx_id -> Bpchar,
        updated_at -> Timestamp,
        topic -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    contents,
    last_status,
    notifies,
    posts,
    transactions,
    users,
);
