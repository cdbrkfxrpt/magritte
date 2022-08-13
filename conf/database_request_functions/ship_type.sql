-- requires one input value: [key]
select shiptype as ship_type
from ais_data.static_ships
where sourcemmsi = $1
limit 1
