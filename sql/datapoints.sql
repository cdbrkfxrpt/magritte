select
	id as "Datapoint ID",
	mmsi as "Vessel ID",
	to_timestamp(ts)::timestamp as "Timestamp",
	lat as "Latitude",
	lon as "Longitude"
from
	ais_data.dynamic_ships
order by
	id
limit 200;
