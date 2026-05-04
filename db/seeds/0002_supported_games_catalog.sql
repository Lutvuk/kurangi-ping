DROP TABLE IF EXISTS _seed_supported_games_rows;

CREATE TEMP TABLE _seed_supported_games_rows (
  game_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  executable_name TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO _seed_supported_games_rows (
  game_id,
  display_name,
  executable_name,
  enabled,
  updated_at
)
VALUES
  ('ffxiv', 'Final Fantasy XIV', 'ffxiv_dx11.exe', 1, '2026-05-05T00:00:00Z'),
  ('valorant', 'VALORANT', 'valorant-win64-shipping.exe', 1, '2026-05-05T00:00:00Z'),
  ('gta_online', 'GTA Online', 'gta5.exe', 1, '2026-05-05T00:00:00Z'),
  ('wow', 'World of Warcraft', 'wow.exe', 1, '2026-05-05T00:00:00Z'),
  ('swtor', 'Star Wars: The Old Republic', 'swtor.exe', 0, '2026-05-05T00:00:00Z');
