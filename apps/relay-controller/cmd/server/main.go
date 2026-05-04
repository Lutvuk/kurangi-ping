package main

import (
	"context"
	"log"
	"net/http"
	"os"

	relaydb "github.com/kurangi-ping/relay-controller/internal/db"
	relayhttp "github.com/kurangi-ping/relay-controller/internal/http"
)

func envOrDefault(key, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}
	return value
}

func main() {
	port := envOrDefault("KP_RELAY_CONTROLLER_PORT", "8080")
	addr := ":" + port
	dbPath := os.Getenv("KP_RELAY_DB_PATH")
	migrationsPath := os.Getenv("KP_RELAY_MIGRATIONS_PATH")

	if dbPath != "" {
		dbConn, err := relaydb.OpenDB(dbPath)
		if err != nil {
			log.Printf("relay-controller db bootstrap failed open: %v", err)
			log.Fatal(err)
		}
		defer dbConn.Close()

		summary, err := relaydb.Migrate(context.Background(), dbConn, migrationsPath)
		if err != nil {
			log.Printf("relay-controller db bootstrap failed migrate: %v", err)
			log.Fatal(err)
		}
		log.Printf(
			"relay-controller db ready path=%s migrations_applied=%d migrations_skipped=%d",
			dbPath,
			summary.AppliedCount,
			summary.SkippedCount,
		)
	} else {
		log.Printf("relay-controller db bootstrap skipped (KP_RELAY_DB_PATH is empty)")
	}

	router := relayhttp.NewRouter()
	log.Printf("relay-controller listening on %s", addr)
	if err := http.ListenAndServe(addr, router); err != nil {
		log.Fatal(err)
	}
}
