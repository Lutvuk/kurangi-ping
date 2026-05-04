package relayhttp

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
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
