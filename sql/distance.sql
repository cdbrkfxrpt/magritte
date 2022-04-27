SELECT ST_Distance(ST_Transform(ship.geom, 3857), ST_Transform(coastline.geom, 3857))
FROM "ais_data"."dynamic_ships" ship, "geographic_features"."europe_coastline" coastline
LIMIT 1
