package relayhttp

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"
	"time"
)

type RelayHealthItem struct {
	RelayID   string `json:"relay_id"`
	Status    string `json:"status"`
	LatencyMS int    `json:"latency_ms"`
	UpdatedAt string `json:"updated_at"`
}

type PageInfo struct {
	NextCursor *string `json:"next_cursor"`
	Limit      int     `json:"limit"`
}

type RelayHealthResponse struct {
	Data []RelayHealthItem `json:"data"`
	Page PageInfo          `json:"page"`
}

func NewRouter() http.Handler {
	mux := http.NewServeMux()
	registerRoutes(mux)
	return withRequestLog(mux)
}

func registerRoutes(mux *http.ServeMux) {
	mux.HandleFunc("GET /v1/relay/health", GetRelayHealth)
}

func GetRelayHealth(w http.ResponseWriter, r *http.Request) {
	limit := parseLimit(r.URL.Query().Get("limit"))

	response := RelayHealthResponse{
		Data: []RelayHealthItem{
			{
				RelayID:   "sin-01",
				Status:    "ok",
				LatencyMS: 38,
				UpdatedAt: time.Now().UTC().Format(time.RFC3339),
			},
			{
				RelayID:   "nrt-01",
				Status:    "warn",
				LatencyMS: 121,
				UpdatedAt: time.Now().UTC().Format(time.RFC3339),
			},
		},
		Page: PageInfo{
			NextCursor: nil,
			Limit:      limit,
		},
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(response)
}

func parseLimit(raw string) int {
	if raw == "" {
		return 50
	}

	limit, err := strconv.Atoi(raw)
	if err != nil {
		return 50
	}

	if limit < 1 {
		return 1
	}
	if limit > 200 {
		return 200
	}
	return limit
}

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (r *statusRecorder) WriteHeader(code int) {
	r.status = code
	r.ResponseWriter.WriteHeader(code)
}

func withRequestLog(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		started := time.Now()
		recorder := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
		next.ServeHTTP(recorder, r)

		log.Printf(
			"request method=%s path=%s status=%d duration_ms=%d",
			r.Method,
			r.URL.Path,
			recorder.status,
			time.Since(started).Milliseconds(),
		)
	})
}
