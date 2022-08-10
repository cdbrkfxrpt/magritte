-- parameters are place in from app config
select {key_name} as "key", {timestamp_name} as "timestamp", {fluent_names}
from {from_table}
order by {order_by} asc
-- offset is used to advance in dataset
offset $1 rows
fetch next {rows_to_fetch} rows only
