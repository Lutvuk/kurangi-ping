package db

import (
	"database/sql"
	"errors"
	"fmt"
	"os"
	"path/filepath"

	_ "modernc.org/sqlite"
)

// OpenDB opens sqlite in local/dev mode and enforces FK pragma.
func OpenDB(path string) (*sql.DB, error) {
	if path == "" {
		return nil, errors.New("sqlite path is required")
	}

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return nil, fmt.Errorf("create sqlite directory: %w", err)
	}

	conn, err := sql.Open("sqlite", path)
	if err != nil {
		return nil, fmt.Errorf("open sqlite connection: %w", err)
	}

	conn.SetMaxOpenConns(1)

	if err := conn.Ping(); err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("ping sqlite connection: %w", err)
	}

	if _, err := conn.Exec("PRAGMA foreign_keys = ON;"); err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("enable sqlite foreign_keys pragma: %w", err)
	}

	var enabled int
	if err := conn.QueryRow("PRAGMA foreign_keys;").Scan(&enabled); err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("query sqlite foreign_keys pragma: %w", err)
	}

	if enabled != 1 {
		_ = conn.Close()
		return nil, errors.New("sqlite foreign_keys pragma is not enabled")
	}

	return conn, nil
}
