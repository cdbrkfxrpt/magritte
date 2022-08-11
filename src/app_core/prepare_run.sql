-- dropping and recreating magritte schema
drop schema if exists magritte cascade;
create schema magritte;

-- creating event stream table
create table magritte.event_stream (
  id          serial,
  fluent_name text,
  keys        integer[],
  timestamp   bigint,
  value       bool,
  lastChanged bigint
);
