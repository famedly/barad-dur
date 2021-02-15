CREATE TABLE IF NOT EXISTS aggregate_stats
(
    day date PRIMARY KEY,
    daily_active_e2ee_rooms BIGINT,
    daily_active_rooms BIGINT,
    daily_active_users BIGINT,
    daily_e2ee_messages BIGINT,
    daily_messages BIGINT,
    daily_sent_e2ee_messages BIGINT,
    daily_sent_messages BIGINT,
    daily_user_type_bridged BIGINT,
    daily_user_type_guest BIGINT,
    daily_user_type_native BIGINT,
    monthly_active_users BIGINT,
    r30_users_all BIGINT,
    r30_users_android BIGINT,
    r30_users_ios BIGINT,
    r30_users_electron BIGINT,
    r30_users_web BIGINT,
    total_nonbridged_users BIGINT,
    total_room_count BIGINT,
    total_users BIGINT,
    daily_active_homeservers BIGINT
);
