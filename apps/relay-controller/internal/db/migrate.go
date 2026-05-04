package db

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type MigrationSummary struct {
	AppliedCount int
	SkippedCount int
}

func Migrate(ctx context.Context, conn *sql.DB, migrationsPath string) (MigrationSummary, error) {
	var summary MigrationSummary
	if conn == nil {
		return summary, fmt.Errorf("sqlite connection is nil")
	}

	if _, err := conn.ExecContext(
		ctx,
		`CREATE TABLE IF NOT EXISTS schema_migrations (
			version TEXT PRIMARY KEY,
			filename TEXT NOT NULL,
			applied_at TEXT NOT NULL DEFAULT (datetime('now'))
		);`,
	); err != nil {
		return summary, fmt.Errorf("create schema_migrations table: %w", err)
	}

	if migrationsPath == "" {
		return summary, nil
	}

	files, err := collectMigrationFiles(migrationsPath)
	if err != nil {
		return summary, err
	}

	applied, err := loadAppliedVersions(ctx, conn)
	if err != nil {
		return summary, err
	}

	for _, fullPath := range files {
		filename := filepath.Base(fullPath)
		version := strings.TrimSuffix(filename, ".up.sql")
		if applied[version] {
			summary.SkippedCount++
			continue
		}

		sqlText, err := os.ReadFile(fullPath)
		if err != nil {
			return summary, fmt.Errorf("read migration %s: %w", filename, err)
		}

		tx, err := conn.BeginTx(ctx, nil)
		if err != nil {
			return summary, fmt.Errorf("begin tx for migration %s: %w", filename, err)
		}

		if _, err := tx.ExecContext(ctx, string(sqlText)); err != nil {
			_ = tx.Rollback()
			return summary, fmt.Errorf("apply migration %s: %w", filename, err)
		}

		if _, err := tx.ExecContext(
			ctx,
			"INSERT INTO schema_migrations(version, filename, applied_at) VALUES (?, ?, datetime('now'))",
			version,
			filename,
		); err != nil {
			_ = tx.Rollback()
			return summary, fmt.Errorf("record migration %s: %w", filename, err)
		}

		if err := tx.Commit(); err != nil {
			return summary, fmt.Errorf("commit migration %s: %w", filename, err)
		}

		summary.AppliedCount++
	}

	return summary, nil
}

func collectMigrationFiles(path string) ([]string, error) {
	entries, err := os.ReadDir(path)
	if err != nil {
		return nil, fmt.Errorf("read migrations directory %s: %w", path, err)
	}

	files := make([]string, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		name := entry.Name()
		if strings.HasSuffix(name, ".up.sql") {
			files = append(files, filepath.Join(path, name))
		}
	}

	sort.Strings(files)
	return files, nil
}

func loadAppliedVersions(ctx context.Context, conn *sql.DB) (map[string]bool, error) {
	rows, err := conn.QueryContext(ctx, "SELECT version FROM schema_migrations")
	if err != nil {
		return nil, fmt.Errorf("query applied migrations: %w", err)
	}
	defer rows.Close()

	result := map[string]bool{}
	for rows.Next() {
		var version string
		if err := rows.Scan(&version); err != nil {
			return nil, fmt.Errorf("scan applied migration version: %w", err)
		}
		result[version] = true
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate applied migration versions: %w", err)
	}

	return result, nil
}
