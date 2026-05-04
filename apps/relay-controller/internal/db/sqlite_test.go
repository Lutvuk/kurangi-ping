package db

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestOpenDBEnablesForeignKeys(t *testing.T) {
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "relay.sqlite")

	conn, err := OpenDB(dbPath)
	if err != nil {
		t.Fatalf("OpenDB error: %v", err)
	}
	defer conn.Close()

	var enabled int
	if err := conn.QueryRow("PRAGMA foreign_keys;").Scan(&enabled); err != nil {
		t.Fatalf("PRAGMA query error: %v", err)
	}

	if enabled != 1 {
		t.Fatalf("expected foreign_keys=1, got %d", enabled)
	}
}

func TestMigrateIsIdempotent(t *testing.T) {
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "relay.sqlite")
	migrationsDir := filepath.Join(tempDir, "migrations")

	if err := os.MkdirAll(migrationsDir, 0o755); err != nil {
		t.Fatalf("mkdir migrations: %v", err)
	}

	if err := os.WriteFile(
		filepath.Join(migrationsDir, "0001_create_alpha.up.sql"),
		[]byte("CREATE TABLE IF NOT EXISTS alpha(id INTEGER PRIMARY KEY);"),
		0o644,
	); err != nil {
		t.Fatalf("write migration 1: %v", err)
	}
	if err := os.WriteFile(
		filepath.Join(migrationsDir, "0002_create_beta.up.sql"),
		[]byte("CREATE TABLE IF NOT EXISTS beta(id INTEGER PRIMARY KEY, alpha_id INTEGER);"),
		0o644,
	); err != nil {
		t.Fatalf("write migration 2: %v", err)
	}

	conn, err := OpenDB(dbPath)
	if err != nil {
		t.Fatalf("OpenDB error: %v", err)
	}
	defer conn.Close()

	first, err := Migrate(context.Background(), conn, migrationsDir)
	if err != nil {
		t.Fatalf("first migrate error: %v", err)
	}
	second, err := Migrate(context.Background(), conn, migrationsDir)
	if err != nil {
		t.Fatalf("second migrate error: %v", err)
	}

	if first.AppliedCount != 2 || first.SkippedCount != 0 {
		t.Fatalf("unexpected first summary: %+v", first)
	}
	if second.AppliedCount != 0 || second.SkippedCount != 2 {
		t.Fatalf("unexpected second summary: %+v", second)
	}
}

func TestMigrateReturnsFileNameOnFailure(t *testing.T) {
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "relay.sqlite")
	migrationsDir := filepath.Join(tempDir, "migrations")

	if err := os.MkdirAll(migrationsDir, 0o755); err != nil {
		t.Fatalf("mkdir migrations: %v", err)
	}

	if err := os.WriteFile(
		filepath.Join(migrationsDir, "0001_broken.up.sql"),
		[]byte("CREATE TABLE broken_sql ("),
		0o644,
	); err != nil {
		t.Fatalf("write broken migration: %v", err)
	}

	conn, err := OpenDB(dbPath)
	if err != nil {
		t.Fatalf("OpenDB error: %v", err)
	}
	defer conn.Close()

	_, err = Migrate(context.Background(), conn, migrationsDir)
	if err == nil {
		t.Fatal("expected migrate error, got nil")
	}

	if want := "0001_broken.up.sql"; !strings.Contains(err.Error(), want) {
		t.Fatalf("expected error to contain %q, got %q", want, err.Error())
	}
}
