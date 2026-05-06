PRAGMA foreign_keys = OFF;

BEGIN;

CREATE TABLE user_settings_new (
  installation_id TEXT PRIMARY KEY,
  preferred_region TEXT CHECK (preferred_region IS NULL OR preferred_region IN ('auto', 'sin', 'nrt', 'us')),
  auto_connect INTEGER NOT NULL CHECK (auto_connect IN (0, 1)),
  update_channel TEXT NOT NULL CHECK (update_channel IN ('beta', 'stable')),
  onboarding_state TEXT NOT NULL,
  updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z')
);

INSERT INTO user_settings_new (
  installation_id,
  preferred_region,
  auto_connect,
  update_channel,
  onboarding_state,
  updated_at
)
SELECT
  installation_id,
  preferred_region,
  auto_connect,
  update_channel,
  onboarding_state,
  updated_at
FROM user_settings;

DROP TABLE user_settings;
ALTER TABLE user_settings_new RENAME TO user_settings;

COMMIT;

PRAGMA foreign_keys = ON;
