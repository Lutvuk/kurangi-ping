package relayhttp

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"reflect"
	"strings"
	"sync"
	"testing"
	"time"
)

func TestGetRelayHealthContractRegression(t *testing.T) {
	withStubProbeExecutor(t, []relayProbeResult{
		{
			RelayID:   "custom-a",
			Success:   true,
			LatencyMS: 777,
			ProbedAt:  time.Date(2026, 9, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "custom-b",
			Success:   false,
			TimedOut:  true,
			ErrorCode: "timeout",
			LatencyMS: 0,
			ProbedAt:  time.Date(2026, 9, 1, 10, 11, 13, 0, time.UTC),
		},
	})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/v1/relay/health?limit=50", nil)
	GetRelayHealth(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rr.Code)
	}

	var root map[string]any
	if err := json.Unmarshal(rr.Body.Bytes(), &root); err != nil {
		t.Fatalf("expected valid json, got error: %v", err)
	}

	// Envelope lock: exactly `data` + `page`.
	if len(root) != 2 {
		t.Fatalf("expected exactly 2 top-level keys, got %d", len(root))
	}
	if _, ok := root["data"]; !ok {
		t.Fatal("missing top-level key: data")
	}
	if _, ok := root["page"]; !ok {
		t.Fatal("missing top-level key: page")
	}

	page, ok := root["page"].(map[string]any)
	if !ok {
		t.Fatalf("expected page object, got %T", root["page"])
	}
	if _, ok := page["next_cursor"]; !ok {
		t.Fatal("missing page.next_cursor")
	}
	if _, ok := page["limit"]; !ok {
		t.Fatal("missing page.limit")
	}

	data, ok := root["data"].([]any)
	if !ok {
		t.Fatalf("expected data array, got %T", root["data"])
	}
	if len(data) != 2 {
		t.Fatalf("expected 2 data entries, got %d", len(data))
	}

	// Item schema lock: exactly relay_id/status/latency_ms/updated_at.
	first, ok := data[0].(map[string]any)
	if !ok {
		t.Fatalf("expected first item object, got %T", data[0])
	}
	if len(first) != 4 {
		t.Fatalf("expected exactly 4 keys on data item, got %d", len(first))
	}
	for _, key := range []string{"relay_id", "status", "latency_ms", "updated_at"} {
		if _, ok := first[key]; !ok {
			t.Fatalf("missing data item key: %s", key)
		}
	}

	// Dynamic source lock: output must reflect probe stub values, not old hardcoded defaults.
	if first["relay_id"] != "custom-a" {
		t.Fatalf("expected dynamic relay_id custom-a, got %#v", first["relay_id"])
	}
	if first["latency_ms"] != float64(777) {
		t.Fatalf("expected dynamic latency_ms 777, got %#v", first["latency_ms"])
	}
	if first["status"] != relayStatusWarn {
		t.Fatalf("expected dynamic status warn for 777ms, got %#v", first["status"])
	}
}

func TestGetRelayHealthShape(t *testing.T) {
	withStubProbeExecutor(t, []relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 37,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "nrt-01",
			Success:   true,
			LatencyMS: 118,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
	})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/v1/relay/health", nil)

	GetRelayHealth(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rr.Code)
	}

	var payload RelayHealthResponse
	if err := json.Unmarshal(rr.Body.Bytes(), &payload); err != nil {
		t.Fatalf("expected valid json, got error: %v", err)
	}

	if len(payload.Data) == 0 {
		t.Fatal("expected at least one relay health item")
	}

	if payload.Page.Limit != 50 {
		t.Fatalf("expected default limit 50, got %d", payload.Page.Limit)
	}

	if payload.Data[0].RelayID != "sin-01" {
		t.Fatalf("expected first relay_id sin-01, got %q", payload.Data[0].RelayID)
	}
	if payload.Data[0].Status != relayStatusOK {
		t.Fatalf("expected first status ok, got %q", payload.Data[0].Status)
	}
	if payload.Data[1].Status != relayStatusWarn {
		t.Fatalf("expected second status warn, got %q", payload.Data[1].Status)
	}
}

func TestGetRelayHealthLimitClamp(t *testing.T) {
	withStubProbeExecutor(t, []relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 20,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "nrt-01",
			Success:   true,
			LatencyMS: 90,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "usw-01",
			Success:   false,
			TimedOut:  true,
			ErrorCode: "timeout",
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
	})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/v1/relay/health?limit=999", nil)

	GetRelayHealth(rr, req)

	var payload RelayHealthResponse
	if err := json.Unmarshal(rr.Body.Bytes(), &payload); err != nil {
		t.Fatalf("expected valid json, got error: %v", err)
	}

	if payload.Page.Limit != 200 {
		t.Fatalf("expected clamped limit 200, got %d", payload.Page.Limit)
	}
	if len(payload.Data) != 3 {
		t.Fatalf("expected all 3 data rows under limit 200, got %d", len(payload.Data))
	}
}

func TestGetRelayHealthAppliesDataLimit(t *testing.T) {
	withStubProbeExecutor(t, []relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 20,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "nrt-01",
			Success:   true,
			LatencyMS: 90,
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
		{
			RelayID:   "usw-01",
			Success:   false,
			TimedOut:  true,
			ErrorCode: "timeout",
			ProbedAt:  time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC),
		},
	})

	rr := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/v1/relay/health?limit=2", nil)

	GetRelayHealth(rr, req)

	var payload RelayHealthResponse
	if err := json.Unmarshal(rr.Body.Bytes(), &payload); err != nil {
		t.Fatalf("expected valid json, got error: %v", err)
	}
	if payload.Page.Limit != 2 {
		t.Fatalf("expected limit 2, got %d", payload.Page.Limit)
	}
	if len(payload.Data) != 2 {
		t.Fatalf("expected payload data to be trimmed by limit, got %d", len(payload.Data))
	}
}

func TestOfflineRelayProbeMapsToDeadStatus(t *testing.T) {
	stubResults := []relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 33,
			ProbedAt:  time.Date(2026, 9, 2, 8, 0, 0, 0, time.UTC),
		},
		{
			RelayID:   "nrt-01",
			Success:   false,
			TimedOut:  true,
			ErrorCode: "timeout",
			LatencyMS: 0,
			ProbedAt:  time.Date(2026, 9, 2, 8, 0, 1, 0, time.UTC),
		},
		{
			RelayID:   "usw-01",
			Success:   false,
			ErrorCode: "network_error",
			LatencyMS: 0,
			ProbedAt:  time.Date(2026, 9, 2, 8, 0, 2, 0, time.UTC),
		},
	}
	withStubProbeExecutor(t, stubResults)

	readPayload := func() RelayHealthResponse {
		rr := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodGet, "/v1/relay/health?limit=50", nil)
		GetRelayHealth(rr, req)

		if rr.Code != http.StatusOK {
			t.Fatalf("expected 200, got %d", rr.Code)
		}

		var payload RelayHealthResponse
		if err := json.Unmarshal(rr.Body.Bytes(), &payload); err != nil {
			t.Fatalf("expected valid json, got error: %v", err)
		}
		return payload
	}

	first := readPayload()
	if len(first.Data) != 3 {
		t.Fatalf("expected full response generation with 3 relays, got %d", len(first.Data))
	}

	statusByRelay := map[string]string{}
	for _, item := range first.Data {
		statusByRelay[item.RelayID] = item.Status
	}
	if statusByRelay["nrt-01"] != relayStatusDead {
		t.Fatalf("expected timeout relay nrt-01 => dead, got %q", statusByRelay["nrt-01"])
	}
	if statusByRelay["usw-01"] != relayStatusDead {
		t.Fatalf("expected network failure relay usw-01 => dead, got %q", statusByRelay["usw-01"])
	}

	second := readPayload()
	if !reflect.DeepEqual(first, second) {
		t.Fatalf("expected deterministic repeated output, first=%+v second=%+v", first, second)
	}
}

func withStubProbeExecutor(t *testing.T, results []relayProbeResult) {
	t.Helper()

	oldExecutor := executeRelayProbes
	oldStore := relayHealthStore

	relayHealthStore = newRelayHealthSnapshotStore()
	executeRelayProbes = func(_ []relayProbeTarget, _ int) []relayProbeResult {
		copied := make([]relayProbeResult, len(results))
		copy(copied, results)
		return copied
	}

	t.Cleanup(func() {
		executeRelayProbes = oldExecutor
		relayHealthStore = oldStore
	})
}

func TestLoadRelayProbeConfigParsesValidTargets(t *testing.T) {
	cfg := loadRelayProbeConfigWithLookup(func(key string) string {
		switch key {
		case envRelayProbeTargets:
			return "sin-01|https://sin-01.example.net/healthz,nrt-01|https://nrt-01.example.net/healthz"
		case envRelayProbeTimeoutMS:
			return "1800"
		default:
			return ""
		}
	})

	wantTargets := []relayProbeTarget{
		{RelayID: "sin-01", ProbeURL: "https://sin-01.example.net/healthz"},
		{RelayID: "nrt-01", ProbeURL: "https://nrt-01.example.net/healthz"},
	}
	if !reflect.DeepEqual(cfg.Targets, wantTargets) {
		t.Fatalf("unexpected parsed targets: got %+v, want %+v", cfg.Targets, wantTargets)
	}
	if cfg.TimeoutMS != 1800 {
		t.Fatalf("expected timeout 1800, got %d", cfg.TimeoutMS)
	}
}

func TestLoadRelayProbeConfigFallsBackWhenTargetsInvalidOrEmpty(t *testing.T) {
	cfgInvalid := loadRelayProbeConfigWithLookup(func(key string) string {
		switch key {
		case envRelayProbeTargets:
			return "bad-format-entry,foo|not-a-url"
		default:
			return ""
		}
	})
	if !reflect.DeepEqual(cfgInvalid.Targets, defaultRelayProbeTargets) {
		t.Fatalf("expected fallback targets for invalid input, got %+v", cfgInvalid.Targets)
	}

	cfgEmpty := loadRelayProbeConfigWithLookup(func(key string) string {
		if key == envRelayProbeTargets {
			return "  "
		}
		return ""
	})
	if !reflect.DeepEqual(cfgEmpty.Targets, defaultRelayProbeTargets) {
		t.Fatalf("expected fallback targets for empty input, got %+v", cfgEmpty.Targets)
	}
}

func TestLoadRelayProbeConfigTimeoutValidationAndClamp(t *testing.T) {
	cases := []struct {
		name   string
		raw    string
		wantMS int
	}{
		{name: "default on empty", raw: "", wantMS: defaultProbeTimeoutMS},
		{name: "default on invalid", raw: "abc", wantMS: defaultProbeTimeoutMS},
		{name: "clamp min", raw: "10", wantMS: minProbeTimeoutMS},
		{name: "clamp max", raw: "60000", wantMS: maxProbeTimeoutMS},
		{name: "pass through valid", raw: "1500", wantMS: 1500},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			cfg := loadRelayProbeConfigWithLookup(func(key string) string {
				switch key {
				case envRelayProbeTargets:
					return ""
				case envRelayProbeTimeoutMS:
					return tc.raw
				default:
					return ""
				}
			})

			if cfg.TimeoutMS != tc.wantMS {
				t.Fatalf("unexpected timeout: got %d, want %d", cfg.TimeoutMS, tc.wantMS)
			}
		})
	}
}

func TestProbeRelayTargetSuccessCapturesLatency(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, `{"ok":true}`)
	}))
	defer server.Close()

	result := probeRelayTarget(
		relayProbeTarget{
			RelayID:  "sin-01",
			ProbeURL: server.URL,
		},
		500,
	)

	if !result.Success {
		t.Fatalf("expected success probe result, got %+v", result)
	}
	if result.HTTPStatus != http.StatusOK {
		t.Fatalf("expected status 200, got %d", result.HTTPStatus)
	}
	if result.LatencyMS < 0 {
		t.Fatalf("expected non-negative latency, got %d", result.LatencyMS)
	}
	if result.ErrorCode != "" {
		t.Fatalf("expected no error code on success, got %q", result.ErrorCode)
	}
}

func TestProbeRelayTargetTimeoutCaptured(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(120 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	result := probeRelayTarget(
		relayProbeTarget{
			RelayID:  "nrt-01",
			ProbeURL: server.URL,
		},
		30,
	)

	if result.Success {
		t.Fatalf("expected timeout failure, got success: %+v", result)
	}
	if !result.TimedOut {
		t.Fatalf("expected timed_out=true, got %+v", result)
	}
	if result.ErrorCode != "timeout" {
		t.Fatalf("expected timeout error code, got %q", result.ErrorCode)
	}
}

func TestProbeRelayTargetNetworkFailureCaptured(t *testing.T) {
	result := probeRelayTarget(
		relayProbeTarget{
			RelayID:  "dead-01",
			ProbeURL: "http://127.0.0.1:1/healthz",
		},
		100,
	)

	if result.Success {
		t.Fatalf("expected network failure, got success: %+v", result)
	}
	if result.ErrorCode != "network_error" {
		t.Fatalf("expected network_error code, got %q", result.ErrorCode)
	}
	if strings.TrimSpace(result.ErrorMessage) == "" {
		t.Fatalf("expected error message to be populated")
	}
}

func TestProbeAllRelaysDeterministicOrder(t *testing.T) {
	client := staticProbeClient{
		responses: map[string]probeStubResponse{
			"http://sin.test/healthz": {statusCode: http.StatusOK},
			"http://nrt.test/healthz": {statusCode: http.StatusServiceUnavailable},
		},
	}

	targets := []relayProbeTarget{
		{RelayID: "sin-01", ProbeURL: "http://sin.test/healthz"},
		{RelayID: "nrt-01", ProbeURL: "http://nrt.test/healthz"},
	}

	var results []relayProbeResult
	for _, target := range targets {
		results = append(results, probeRelayTargetWithClient(client, target, 200))
	}

	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}
	if results[0].RelayID != "sin-01" || results[1].RelayID != "nrt-01" {
		t.Fatalf("expected deterministic result order by input, got %+v", results)
	}
	if !results[0].Success {
		t.Fatalf("expected first probe success, got %+v", results[0])
	}
	if results[1].ErrorCode != "http_status" {
		t.Fatalf("expected second probe http_status failure, got %+v", results[1])
	}
}

func TestProbeAllRelaysUsesRealExecutor(t *testing.T) {
	serverA := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer serverA.Close()
	serverB := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer serverB.Close()

	results := probeAllRelays([]relayProbeTarget{
		{RelayID: "sin-01", ProbeURL: serverA.URL},
		{RelayID: "nrt-01", ProbeURL: serverB.URL},
	}, 300)

	if len(results) != 2 {
		t.Fatalf("expected 2 probe results, got %d", len(results))
	}
	if !results[0].Success || !results[1].Success {
		t.Fatalf("expected all probes to succeed, got %+v", results)
	}
}

func TestClassifyRelayStatus(t *testing.T) {
	cases := []struct {
		name  string
		input relayProbeResult
		want  string
	}{
		{
			name: "success low latency maps ok",
			input: relayProbeResult{
				Success:   true,
				LatencyMS: 38,
			},
			want: relayStatusOK,
		},
		{
			name: "success high latency maps warn",
			input: relayProbeResult{
				Success:   true,
				LatencyMS: 120,
			},
			want: relayStatusWarn,
		},
		{
			name: "success boundary 50ms maps warn",
			input: relayProbeResult{
				Success:   true,
				LatencyMS: 50,
			},
			want: relayStatusWarn,
		},
		{
			name: "timeout maps dead",
			input: relayProbeResult{
				Success:   false,
				TimedOut:  true,
				ErrorCode: "timeout",
			},
			want: relayStatusDead,
		},
		{
			name: "network failure maps dead",
			input: relayProbeResult{
				Success:   false,
				ErrorCode: "network_error",
			},
			want: relayStatusDead,
		},
		{
			name: "http failure maps dead",
			input: relayProbeResult{
				Success:    false,
				HTTPStatus: http.StatusServiceUnavailable,
				ErrorCode:  "http_status",
			},
			want: relayStatusDead,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := classifyRelayStatus(tc.input)
			if got != tc.want {
				t.Fatalf("unexpected status mapping: got %q, want %q", got, tc.want)
			}
		})
	}
}

func TestRelayHealthSnapshotStoreUpdateAndRead(t *testing.T) {
	store := newRelayHealthSnapshotStore()
	now := time.Date(2026, 8, 1, 10, 11, 12, 0, time.UTC)
	store.UpdateFromProbeResults([]relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 42,
			ProbedAt:  now,
		},
		{
			RelayID:   "nrt-01",
			Success:   true,
			LatencyMS: 120,
			ProbedAt:  now,
		},
	})

	got := store.Read()
	if len(got) != 2 {
		t.Fatalf("expected 2 items, got %d", len(got))
	}
	if got[0].Status != relayStatusOK {
		t.Fatalf("expected first status ok, got %q", got[0].Status)
	}
	if got[1].Status != relayStatusWarn {
		t.Fatalf("expected second status warn, got %q", got[1].Status)
	}
	if got[0].UpdatedAt != now.Format(time.RFC3339) {
		t.Fatalf("unexpected updated_at: got %q", got[0].UpdatedAt)
	}
}

func TestRelayHealthSnapshotStoreReadReturnsCopy(t *testing.T) {
	store := newRelayHealthSnapshotStore()
	store.UpdateFromProbeResults([]relayProbeResult{
		{
			RelayID:   "sin-01",
			Success:   true,
			LatencyMS: 38,
			ProbedAt:  time.Date(2026, 8, 2, 1, 2, 3, 0, time.UTC),
		},
	})

	first := store.Read()
	if len(first) != 1 {
		t.Fatalf("expected 1 item, got %d", len(first))
	}
	first[0].Status = relayStatusDead

	second := store.Read()
	if second[0].Status != relayStatusOK {
		t.Fatalf("expected read result to be copy-safe, got %q", second[0].Status)
	}
}

func TestRelayHealthSnapshotStoreConcurrentReadWrite(t *testing.T) {
	store := newRelayHealthSnapshotStore()

	var wg sync.WaitGroup
	for i := 0; i < 25; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			store.UpdateFromProbeResults([]relayProbeResult{
				{
					RelayID:   "sin-01",
					Success:   true,
					LatencyMS: idx + 30,
					ProbedAt:  time.Now().UTC(),
				},
				{
					RelayID:   "nrt-01",
					Success:   false,
					TimedOut:  true,
					ErrorCode: "timeout",
					LatencyMS: 0,
					ProbedAt:  time.Now().UTC(),
				},
			})
		}(i)
	}

	for i := 0; i < 25; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_ = store.Read()
		}()
	}

	wg.Wait()
	final := store.Read()
	if len(final) != 2 {
		t.Fatalf("expected 2 items in final snapshot, got %d", len(final))
	}
	if final[1].Status != relayStatusDead {
		t.Fatalf("expected timed-out relay to map dead, got %q", final[1].Status)
	}
}

type probeStubResponse struct {
	statusCode int
	err        error
}

type staticProbeClient struct {
	responses map[string]probeStubResponse
}

func (c staticProbeClient) Do(req *http.Request) (*http.Response, error) {
	entry, ok := c.responses[req.URL.String()]
	if !ok {
		return nil, errors.New("missing stub")
	}
	if entry.err != nil {
		return nil, entry.err
	}
	return &http.Response{
		StatusCode: entry.statusCode,
		Status:     fmt.Sprintf("%d %s", entry.statusCode, http.StatusText(entry.statusCode)),
		Body:       io.NopCloser(strings.NewReader("ok")),
		Header:     make(http.Header),
	}, nil
}
