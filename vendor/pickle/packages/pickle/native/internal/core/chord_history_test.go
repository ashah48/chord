package core

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"maunium.net/go/mautrix"
	"maunium.net/go/mautrix/id"
)

func TestChordHistoryKeepsThreadReplies(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		if r.URL.Path == "/_matrix/client/v3/rooms/!room:example/messages" {
			_, _ = w.Write([]byte(`{"chunk":[{"event_id":"$reply","type":"m.room.message","sender":"@bob:example","origin_server_ts":1000,"content":{"msgtype":"m.text","body":"A reply that must remain visible","m.relates_to":{"rel_type":"m.thread","event_id":"$root","m.in_reply_to":{"event_id":"$root"}}}}],"end":"older"}`))
			return
		}
		_, _ = w.Write([]byte(`{"chunk":[]}`))
	}))
	defer server.Close()
	core := New(nil)
	core.client, _ = mautrix.NewClient(server.URL, id.UserID("@alice:example"), "local-test")
	raw, err := core.handleFetchMessages(context.Background(), []byte(`{"roomId":"!room:example","limit":50}`))
	if err != nil {
		t.Fatal(err)
	}
	var result struct {
		Messages []*MatrixMessageEvent `json:"messages"`
		NextCursor string `json:"nextCursor"`
	}
	if err := json.Unmarshal(raw, &result); err != nil {
		t.Fatal(err)
	}
	if len(result.Messages) != 1 || result.Messages[0].ThreadRootEventID == nil || result.NextCursor != "older" {
		t.Fatalf("thread reply or pagination lost: %s", raw)
	}
}
