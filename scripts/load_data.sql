CREATE TABLE players AS
SELECT unnest(map_values(json), recursive := true)
FROM read_json('data/raw/players.json');
