-- transforming coastline to the correct coordinate system:
-- this is very costly but can be done only once on startup
select gid, shape_leng, ST_Transform(geom, 3857) as geom
into   magritte.europe_coastline
from   geographic_features.europe_coastline;
