select
	id,
	mmsi as "source",
	to_timestamp(ts)::timestamp as "timestamp",
	lat,
	lon,
  speed
from ais_data.dynamic_ships
order by id asc
offset $1 rows
next {} rows only
