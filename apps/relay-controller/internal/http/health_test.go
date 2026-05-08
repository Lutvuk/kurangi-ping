package relayhttp

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"reflect"
	"testing"
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
