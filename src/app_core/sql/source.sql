-- parameters are placed in from app config
select {key_name} as "key", {timestamp_name} as "timestamp", {fluent_names}
from {from_table}
where {timestamp_name} between {start_time} and {end_time}
order by {timestamp_name} desc, {order_by} desc
