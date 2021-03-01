-- Add migration script here
CREATE TABLE IF NOT EXISTS StatsReport
(
    id BIGSERIAL PRIMARY KEY,
    local_timestamp timestamp with time zone,
    remote_timestamp timestamp with time zone,
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
    cpu_average BIGINT,
    event_cache_size BIGINT,
    memory_rss BIGINT,
    monthly_active_users BIGINT,
    r30_users_all BIGINT,
    r30_users_android BIGINT,
    r30_users_ios BIGINT,
    r30_users_electron BIGINT,
    r30_users_web BIGINT,
    total_nonbridged_users BIGINT,
    total_room_count BIGINT,
    total_users BIGINT,
    uptime_seconds BIGINT,
    cache_factor DOUBLE PRECISION,
    database_engine TEXT,
    database_server_version TEXT,
    homeserver TEXT,
    log_level TEXT,
    python_version TEXT,
    server_context TEXT,
    remote_addr TEXT,
    x_forwarded_for TEXT,
    user_agent TEXT
);
