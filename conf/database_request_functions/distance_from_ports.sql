-- requires two input values: [lon, lat]
select distance
from (
  select ST_Distance(
    ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
    ST_Transform(geom, 3857)
  ) as distance
  from
    ports.ports_of_brittany
) subquery
where
  subquery.distance <= 3000.0
