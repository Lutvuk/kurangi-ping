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
	"testing"
	"time"
)

func TestGetRelayHealthShape(t *testing.T) {
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
}

func TestGetRelayHealthLimitClamp(t *testing.T) {
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
