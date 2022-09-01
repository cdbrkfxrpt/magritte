-- parameters are placed in from app config
select {key_name} as "key", {timestamp_name} as "timestamp", {fluent_names}
from {from_table}
where {timestamp_name} = $1
order by {order_by} asc
