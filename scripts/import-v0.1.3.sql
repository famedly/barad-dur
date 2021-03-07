CREATE TABLE IF NOT EXISTS StatsReportStaging
(
    id BIGSERIAL PRIMARY KEY,
    homeserver TEXT,
    local_timestamp BIGINT,
    remote_timestamp BIGINT,
    remote_addr TEXT,
    forwarded_for TEXT,
    uptime_seconds BIGINT,
    total_users BIGINT,
    total_nonbridged_users BIGINT,
    total_room_count BIGINT,
    daily_active_users BIGINT,
    daily_active_rooms BIGINT,
    daily_messages BIGINT,
    daily_sent_messages BIGINT,
    daily_active_e2ee_rooms BIGINT,
    daily_e2ee_messages BIGINT,
    daily_sent_e2ee_messages BIGINT,
    monthly_active_users BIGINT,
    r30_users_all BIGINT,
    r30_users_android BIGINT,
    r30_users_ios BIGINT,
    r30_users_electron BIGINT,
    r30_users_web BIGINT,
    cpu_average BIGINT,
    memory_rss BIGINT,
    cache_factor TEXT,
    event_cache_size BIGINT,
    user_agent TEXT,
    daily_user_type_native BIGINT,
    daily_user_type_bridged BIGINT,
    daily_user_type_guest BIGINT,
    python_version TEXT,
    database_engine TEXT,
    database_server_version TEXT,
    server_context TEXT,
    log_level TEXT
);

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


\copy StatsReportStaging (id, homeserver, local_timestamp, remote_timestamp, remote_addr, x_forwarded_for, uptime_seconds, total_users,total_nonbridged_users, total_room_count, daily_active_users,daily_active_rooms, daily_messages, daily_sent_messages, monthly_active_users, r30_users_all, r30_users_android, r30_users_ios, r30_users_electron,r30_users_web, cpu_average, memory_rss, cache_factor,event_cache_size, user_agent, daily_user_type_native, daily_user_type_bridged, daily_user_type_guest, python_version, database_engine, database_server_version, server_context, log_level) FROM pstdin WITH DELIMITER ',' NULL '\N' CSV;
ALTER TABLE StatsReportStaging
    ALTER COLUMN local_timestamp SET DATA TYPE timestamp with time zone
    USING to_timestamp(local_timestamp);

ALTER TABLE StatsReportStaging
    ALTER COLUMN remote_timestamp SET DATA TYPE timestamp with time zone
    USING to_timestamp(remote_timestamp);

ALTER TABLE StatsReportStaging
    ALTER COLUMN cache_factor SET DATA TYPE double precision
    USING cache_factor::double precision;


INSERT INTO StatsReport 
(
    local_timestamp,
    remote_timestamp,
    daily_active_e2ee_rooms,
    daily_active_rooms,
    daily_active_users,
    daily_e2ee_messages,
    daily_messages,
    daily_sent_e2ee_messages,
    daily_sent_messages,
    daily_user_type_bridged,
    daily_user_type_guest,
    daily_user_type_native,
    cpu_average,
    event_cache_size,
    memory_rss,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    total_nonbridged_users,
    total_room_count,
    total_users,
    uptime_seconds,
    cache_factor,
    database_engine,
    database_server_version,
    homeserver,
    log_level,
    python_version,
    server_context,
    remote_addr,
    x_forwarded_for,
    user_agent
) SELECT
    local_timestamp,
    remote_timestamp,
    daily_active_e2ee_rooms,
    daily_active_rooms,
    daily_active_users,
    daily_e2ee_messages,
    daily_messages,
    daily_sent_e2ee_messages,
    daily_sent_messages,
    daily_user_type_bridged,
    daily_user_type_guest,
    daily_user_type_native,
    cpu_average,
    event_cache_size,
    memory_rss,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    total_nonbridged_users,
    total_room_count,
    total_users,
    uptime_seconds,
    cache_factor,
    database_engine,
    database_server_version,
    homeserver,
    log_level,
    python_version,
    server_context,
    remote_addr,
    x_forwarded_for,
    user_agent
FROM StatsReportStaging;


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


INSERT INTO aggregate_stats (
    day,
    daily_active_e2ee_rooms,
    daily_active_rooms,
    daily_active_users,
    daily_e2ee_messages,
    daily_messages,
    daily_sent_e2ee_messages,
    daily_sent_messages,
    daily_user_type_bridged,
    daily_user_type_guest,
    daily_user_type_native,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    total_nonbridged_users,
    total_room_count,
    total_users,
    daily_active_homeservers
)
SELECT
    local_timestamp::DATE,
    SUM(daily_active_e2ee_rooms),
    SUM(daily_active_rooms),
    SUM(daily_active_users),
    SUM(daily_e2ee_messages),
    SUM(daily_messages),
    SUM(daily_sent_e2ee_messages),
    SUM(daily_sent_messages),
    SUM(daily_user_type_bridged),
    SUM(daily_user_type_guest),
    SUM(daily_user_type_native),
    SUM(monthly_active_users),
    SUM(r30_users_all),
    SUM(r30_users_android),
    SUM(r30_users_ios),
    SUM(r30_users_electron),
    SUM(r30_users_web),
    SUM(total_nonbridged_users),
    SUM(total_room_count),
    SUM(total_users),
    COUNT(homeserver)
FROM statsreport
GROUP BY local_timestamp::DATE;

DROP TABLE StatsReportStaging;