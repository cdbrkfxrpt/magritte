-- requires two input values: [lon, lat]
select ST_Distance(
  ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
  geom
) as distance
from
  magritte.europe_coastline
limit 1
