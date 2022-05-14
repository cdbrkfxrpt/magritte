select
	id,
	mmsi as "source",
  ts as "timestamp",
	lon,
	lat,
  speed
from ais_data.dynamic_ships
order by id asc
offset $1 rows
fetch next {} rows only
