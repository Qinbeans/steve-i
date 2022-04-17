table! {
    guilds (id) {
        id -> Bigint,
        name -> Nullable<Varchar>,
        prefix -> Nullable<Varchar>,
        owner_id -> Nullable<Bigint>,
        cur_vc_id -> Nullable<Varchar>,
    }
}

table! {
    guild_users (user_id, guild_id) {
        user_id -> Bigint,
        guild_id -> Bigint,
    }
}

table! {
    users (id) {
        id -> Bigint,
        name -> Nullable<Varchar>,
        tag -> Nullable<Varchar>,
        current_channel_id -> Nullable<Longtext>,
    }
}

joinable!(guild_users -> guilds (guild_id));
joinable!(guild_users -> users (user_id));
joinable!(guilds -> users (owner_id));

allow_tables_to_appear_in_same_query!(
    guilds,
    guild_users,
    users,
);
