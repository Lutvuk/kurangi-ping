package relayhttp

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"log"
	"net"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
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

type relayProbeTarget struct {
	RelayID  string
	ProbeURL string
}

type relayProbeConfig struct {
	Targets   []relayProbeTarget
	TimeoutMS int
}

type relayProbeResult struct {
	RelayID      string
	ProbeURL     string
	LatencyMS    int
	HTTPStatus   int
	Success      bool
	TimedOut     bool
	ErrorCode    string
	ErrorMessage string
	ProbedAt     time.Time
}

type relayHTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

const (
	envRelayProbeTargets   = "KP_RELAY_PROBE_TARGETS"
	envRelayProbeTimeoutMS = "KP_RELAY_PROBE_TIMEOUT_MS"
	defaultProbeTimeoutMS  = 1200
	minProbeTimeoutMS      = 100
	maxProbeTimeoutMS      = 10000
	relayStatusOK          = "ok"
	relayStatusWarn        = "warn"
	relayStatusDead        = "dead"
	relayLatencyOKMaxMS    = 49
)

var defaultRelayProbeTargets = []relayProbeTarget{
	{RelayID: "sin-01", ProbeURL: "https://sin-01.example.net/healthz"},
	{RelayID: "nrt-01", ProbeURL: "https://nrt-01.example.net/healthz"},
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
	_ = loadRelayProbeConfig()
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

func loadRelayProbeConfig() relayProbeConfig {
	return loadRelayProbeConfigWithLookup(os.Getenv)
}

func loadRelayProbeConfigWithLookup(getenv func(string) string) relayProbeConfig {
	targets := parseRelayProbeTargets(getenv(envRelayProbeTargets))
	if len(targets) == 0 {
		targets = make([]relayProbeTarget, len(defaultRelayProbeTargets))
		copy(targets, defaultRelayProbeTargets)
	}

	timeout := parseProbeTimeoutMS(getenv(envRelayProbeTimeoutMS))
	return relayProbeConfig{
		Targets:   targets,
		TimeoutMS: timeout,
	}
}

func probeAllRelays(targets []relayProbeTarget, timeoutMS int) []relayProbeResult {
	results := make([]relayProbeResult, 0, len(targets))
	for _, target := range targets {
		results = append(results, probeRelayTarget(target, timeoutMS))
	}
	return results
}

func probeRelayTarget(target relayProbeTarget, timeoutMS int) relayProbeResult {
	client := &http.Client{}
	return probeRelayTargetWithClient(client, target, timeoutMS)
}

func probeRelayTargetWithClient(
	client relayHTTPClient,
	target relayProbeTarget,
	timeoutMS int,
) relayProbeResult {
	startedAt := time.Now().UTC()
	timeout := time.Duration(parseProbeTimeoutMS(strconv.Itoa(timeoutMS))) * time.Millisecond

	result := relayProbeResult{
		RelayID:  target.RelayID,
		ProbeURL: target.ProbeURL,
		ProbedAt: startedAt,
	}

	req, err := http.NewRequestWithContext(
		context.Background(),
		http.MethodGet,
		target.ProbeURL,
		nil,
	)
	if err != nil {
		result.ErrorCode = "invalid_target_url"
		result.ErrorMessage = err.Error()
		return result
	}

	ctx, cancel := context.WithTimeout(req.Context(), timeout)
	defer cancel()
	req = req.WithContext(ctx)

	resp, err := client.Do(req)
	result.LatencyMS = int(time.Since(startedAt).Milliseconds())
	if result.LatencyMS < 0 {
		result.LatencyMS = 0
	}

	if err != nil {
		result.ErrorMessage = err.Error()
		if errors.Is(err, context.DeadlineExceeded) || isTimeoutError(err) {
			result.TimedOut = true
			result.ErrorCode = "timeout"
		} else {
			result.ErrorCode = "network_error"
		}
		return result
	}
	defer resp.Body.Close()
	_, _ = io.Copy(io.Discard, resp.Body)

	result.HTTPStatus = resp.StatusCode
	if resp.StatusCode >= http.StatusOK && resp.StatusCode < http.StatusBadRequest {
		result.Success = true
		return result
	}

	result.ErrorCode = "http_status"
	result.ErrorMessage = http.StatusText(resp.StatusCode)
	return result
}

func classifyRelayStatus(result relayProbeResult) string {
	if !result.Success {
		return relayStatusDead
	}

	if result.LatencyMS <= relayLatencyOKMaxMS {
		return relayStatusOK
	}

	return relayStatusWarn
}

func parseRelayProbeTargets(raw string) []relayProbeTarget {
	if strings.TrimSpace(raw) == "" {
		return nil
	}

	entries := strings.Split(raw, ",")
	parsed := make([]relayProbeTarget, 0, len(entries))
	for _, entry := range entries {
		pair := strings.SplitN(strings.TrimSpace(entry), "|", 2)
		if len(pair) != 2 {
			continue
		}

		relayID := strings.TrimSpace(pair[0])
		probeURL := strings.TrimSpace(pair[1])
		if relayID == "" || !isValidProbeURL(probeURL) {
			continue
		}

		parsed = append(parsed, relayProbeTarget{
			RelayID:  relayID,
			ProbeURL: probeURL,
		})
	}

	return parsed
}

func parseProbeTimeoutMS(raw string) int {
	if strings.TrimSpace(raw) == "" {
		return defaultProbeTimeoutMS
	}

	parsed, err := strconv.Atoi(raw)
	if err != nil {
		return defaultProbeTimeoutMS
	}

	if parsed < minProbeTimeoutMS {
		return minProbeTimeoutMS
	}
	if parsed > maxProbeTimeoutMS {
		return maxProbeTimeoutMS
	}
	return parsed
}

func isValidProbeURL(raw string) bool {
	parsed, err := url.Parse(raw)
	if err != nil {
		return false
	}

	if parsed.Host == "" {
		return false
	}

	return parsed.Scheme == "https" || parsed.Scheme == "http"
}

func isTimeoutError(err error) bool {
	var netErr net.Error
	return errors.As(err, &netErr) && netErr.Timeout()
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
