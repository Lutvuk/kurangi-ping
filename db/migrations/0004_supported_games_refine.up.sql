PRAGMA foreign_keys = OFF;

BEGIN;

CREATE TABLE supported_games_new (
  game_id TEXT PRIMARY KEY CHECK (
    game_id = lower(trim(game_id))
    AND game_id GLOB '[a-z0-9_]*'
    AND length(game_id) BETWEEN 2 AND 64
  ),
  display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 2 AND 120),
  executable_name TEXT NOT NULL UNIQUE CHECK (
    executable_name = lower(trim(executable_name))
    AND executable_name LIKE '%.exe'
    AND instr(executable_name, '/') = 0
    AND instr(executable_name, '\') = 0
    AND length(executable_name) BETWEEN 5 AND 128
  ),
  match_mode TEXT NOT NULL DEFAULT 'exact' CHECK (match_mode IN ('exact')),
  catalog_version TEXT NOT NULL DEFAULT 'v1' CHECK (catalog_version GLOB 'v[0-9]*'),
  enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
  updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z')
);

INSERT INTO supported_games_new (
  game_id, display_name, executable_name, match_mode, catalog_version, enabled, updated_at
)
SELECT
  lower(trim(game_id)),
  trim(display_name),
  lower(trim(executable_name)),
  'exact',
  'v1',
  enabled,
  updated_at
FROM supported_games;

DROP TABLE supported_games;
ALTER TABLE supported_games_new RENAME TO supported_games;

CREATE INDEX IF NOT EXISTS idx_supported_games_enabled_executable
  ON supported_games(enabled, executable_name);

COMMIT;

PRAGMA foreign_keys = ON;
