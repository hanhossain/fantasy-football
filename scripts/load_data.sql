CREATE TABLE players AS
SELECT unnest(map_values(json), recursive := true)
FROM 'data/raw/players.json';

CREATE TABLE schedules AS
SELECT *, parse_path(filename)[4] AS season_type, parse_filename(filename, true) AS season
FROM 'data/raw/schedules/*/*.json';
