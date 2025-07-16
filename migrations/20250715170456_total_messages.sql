-- Add migration script here
ALTER TABLE aggregated_stats
  ADD total_messages BIGINT,
  ADD total_e2ee_messages BIGINT;

ALTER TABLE aggregated_stats_by_context
  ADD total_messages BIGINT,
  ADD total_e2ee_messages BIGINT;