CREATE TABLE StatsReportStaging
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

\copy StatsReportStaging ( id, homeserver, local_timestamp, remote_timestamp, remote_addr, forwarded_for, uptime_seconds, total_users, total_nonbridged_users, total_room_count, daily_active_users, daily_active_rooms, daily_messages, daily_sent_messages, monthly_active_users, r30_users_all, r30_users_android, r30_users_ios, r30_users_electron, r30_users_web, cpu_average, memory_rss, cache_factor, event_cache_size, user_agent, daily_user_type_native, daily_user_type_bridged, daily_user_type_guest, python_version, database_engine, database_server_version, server_context, log_leve ) FROM pstdin WITH DELIMITER ',' NULL '\N' CSV;

ALTER TABLE StatsReportStaging
    ALTER COLUMN local_timestamp SET DATA TYPE timestamp with time zone
    USING to_timestamp(local_timestamp);

ALTER TABLE StatsReportStaging
    ALTER COLUMN remote_timestamp SET DATA TYPE timestamp with time zone
    USING to_timestamp(remote_timestamp);

ALTER TABLE StatsReportStaging
    ALTER COLUMN cache_factor SET DATA TYPE double precision
    USING cache_factor::double precision;


INSERT INTO reports
(
    homeserver,
    local_timestamp,
    remote_timestamp,
    remote_addr,
    forwarded_for,
    uptime_seconds,
    total_users,
    total_nonbridged_users,
    total_room_count,
    daily_active_users,
    daily_active_rooms,
    daily_messages,
    daily_sent_messages,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    cpu_average,
    memory_rss,
    cache_factor,
    event_cache_size,
    user_agent,
    daily_user_type_native,
    daily_user_type_bridged,
    daily_user_type_guest,
    python_version,
    database_engine,
    database_server_version,
    server_context,
    log_level
) SELECT
    homeserver,
    local_timestamp,
    remote_timestamp,
    remote_addr,
    forwarded_for,
    uptime_seconds,
    total_users,
    total_nonbridged_users,
    total_room_count,
    daily_active_users,
    daily_active_rooms,
    daily_messages,
    daily_sent_messages,
    monthly_active_users,
    r30_users_all,
    r30_users_android,
    r30_users_ios,
    r30_users_electron,
    r30_users_web,
    cpu_average,
    memory_rss,
    cache_factor,
    event_cache_size,
    user_agent,
    daily_user_type_native,
    daily_user_type_bridged,
    daily_user_type_guest,
    python_version,
    database_engine,
    database_server_version,
    server_context,
    log_level
FROM StatsReportStaging;

DELETE FROM aggregated_stats;

DROP TABLE StatsReportStaging;
