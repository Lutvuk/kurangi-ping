package main

import (
	"log"
	"net/http"
	"os"

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

	router := relayhttp.NewRouter()
	log.Printf("relay-controller listening on %s", addr)
	if err := http.ListenAndServe(addr, router); err != nil {
		log.Fatal(err)
	}
}
