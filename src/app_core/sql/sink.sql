-- inserting into event stream table...
insert into
  magritte.event_stream (
    fluent_name,
    keys,
    timestamp,
    value,
    last_change
  )
-- ... using the following variables.
-- variables are replaced by values.
values
  ($1, $2, $3, $4,  $5)
