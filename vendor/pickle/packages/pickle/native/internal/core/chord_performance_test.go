package core

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"maunium.net/go/mautrix"
	"maunium.net/go/mautrix/event"
	"maunium.net/go/mautrix/id"
)

// Include network latency: an in-process server alone hides serialized I/O.
func BenchmarkChordHistory50(b *testing.B) {
	events := make([]map[string]any, 50)
	for i := range events {
		events[i] = map[string]any{"event_id": fmt.Sprintf("$%d", i), "type": "m.room.message", "sender": "@alice:example", "origin_server_ts": i, "content": map[string]any{"msgtype": "m.text", "body": "message"}}
	}
	page, _ := json.Marshal(map[string]any{"chunk": events, "end": "next"})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		time.Sleep(2 * time.Millisecond)
		if r.URL.Path == "/_matrix/client/v3/rooms/!room:example/messages" {
			w.Write(page)
		} else {
			w.Write([]byte(`{"chunk":[]}`))
		}
	}))
	defer server.Close()
	core := New(nil)
	core.client, _ = mautrix.NewClient(server.URL, id.UserID("@me:example"), "test")
	payload := []byte(`{"roomId":"!room:example","limit":50}`)
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, err := core.handleFetchMessages(context.Background(), payload); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkChordUnstoredState(b *testing.B) {
	store, err := newPersistentStateStore(context.Background(), memoryByteStore{values: map[string][]byte{}}, "state")
	if err != nil {
		b.Fatal(err)
	}
	members := make(map[id.UserID]*event.MemberEventContent, 1000)
	for i := 0; i < 1000; i++ {
		members[id.UserID(fmt.Sprintf("@user%d:example", i))] = &event.MemberEventContent{Membership: event.MembershipJoin, Displayname: "Member"}
	}
	store.Members["!room:example"] = members
	empty := ""
	evt := &event.Event{Type: event.StateRoomName, RoomID: "!room:example", StateKey: &empty, Content: event.Content{Parsed: &event.RoomNameEventContent{Name: "New name"}}}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		store.UpdateState(context.Background(), evt)
	}
}

func TestChordReplacementPoolPreservesMessages(t *testing.T) {
	var active, peak, calls atomic.Int32
	ready := make(chan struct{})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		calls.Add(1)
		n := active.Add(1)
		defer active.Add(-1)
		for old := peak.Load(); n > old; old = peak.Load() {
			if peak.CompareAndSwap(old, n) {
				break
			}
		}
		if n == replacementFetchConcurrency {
			select {
			case <-ready:
			default:
				close(ready)
			}
		}
		select {
		case <-ready:
		case <-r.Context().Done():
			return
		case <-time.After(3 * time.Second):
			t.Error("edit lookups did not run concurrently")
			return
		}
		w.Header().Set("Content-Type", "application/json")
		if strings.Contains(r.URL.Path, "/$fail/") {
			w.WriteHeader(http.StatusForbidden)
			w.Write([]byte(`{"errcode":"M_FORBIDDEN","error":"no access"}`))
			return
		}
		if strings.Contains(r.URL.Path, "/$0/") {
			w.Write([]byte(`{"chunk":[{"event_id":"$edit","type":"m.room.message","sender":"@alice:example","origin_server_ts":99,"content":{"msgtype":"m.text","body":"* edited","m.new_content":{"msgtype":"m.text","body":"edited"},"m.relates_to":{"rel_type":"m.replace","event_id":"$0"}}}]}`))
		} else {
			w.Write([]byte(`{"chunk":[]}`))
		}
	}))
	defer server.Close()
	core := New(nil)
	core.client, _ = mautrix.NewClient(server.URL, id.UserID("@me:example"), "test")
	var messages []*MatrixMessageEvent
	for i := 0; i < 9; i++ {
		messages = append(messages, &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: fmt.Sprintf("$%d", i)}, Body: "original"})
	}
	messages[0].setThreadRoot("$thread")
	messages = append(messages, messages[0], &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: "$fail"}, Body: "keep on failure"})
	cached := &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: "$cached"}, Body: "cached edit"}
	core.messageEdits["$cached"] = cached
	messages = append(messages, &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: "$cached"}})
	edited := true
	messages = append(messages, &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: "$edited"}, IsEdited: &edited})
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	result := core.resolveMessageReplacements(ctx, core.client, "!room:example", messages)
	if ctx.Err() != nil {
		t.Fatal(ctx.Err())
	}
	if peak.Load() != replacementFetchConcurrency {
		t.Fatalf("peak lookups = %d", peak.Load())
	}
	if calls.Load() != 10 {
		t.Fatalf("expected 10 unique uncached lookups, got %d", calls.Load())
	}
	if len(result) != 12 || result[0].Body != "edited" || stringValue(result[0].ThreadRootEventID) != "$thread" || result[9].Body != "keep on failure" || result[10] != cached || result[11].EventID != "$edited" {
		t.Fatalf("messages changed or lost: %#v", result)
	}
	for i := 1; i < 9; i++ {
		if result[i].EventID != fmt.Sprintf("$%d", i) {
			t.Fatalf("order changed at %d", i)
		}
	}
}

func TestChordCancelledReplacementPoolDoesNotFetch(t *testing.T) {
	core := New(nil)
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	msg := &MatrixMessageEvent{MatrixRawEvent: MatrixRawEvent{EventID: "$message"}}
	// A cancelled page never starts HTTP work, even with no client configured.
	result := core.resolveMessageReplacements(ctx, nil, "!room:example", []*MatrixMessageEvent{msg})
	if len(result) != 1 || result[0] != msg {
		t.Fatal("cancelled lookup discarded message")
	}
}

type countingByteStore struct {
	memoryByteStore
	writes int
}

func (store *countingByteStore) Set(ctx context.Context, key string, value []byte) error {
	store.writes++
	return store.memoryByteStore.Set(ctx, key, value)
}

func TestChordStateOnlyPersistsStoredEvents(t *testing.T) {
	ctx := context.Background()
	kv := &countingByteStore{memoryByteStore: memoryByteStore{values: map[string][]byte{}}}
	store, err := newPersistentStateStore(ctx, kv, "state")
	if err != nil {
		t.Fatal(err)
	}
	empty, user := "", "@alice:example"
	for _, evt := range []*event.Event{
		nil,
		{Type: event.EventMessage, Content: event.Content{Parsed: &event.MessageEventContent{Body: "hi"}}},
		{Type: event.StateRoomName, StateKey: &empty, Content: event.Content{Parsed: &event.RoomNameEventContent{Name: "room"}}},
		{Type: event.StatePowerLevels, StateKey: &user, Content: event.Content{Parsed: &event.PowerLevelsEventContent{}}},
	} {
		store.UpdateState(ctx, evt)
	}
	if kv.writes != 0 {
		t.Fatalf("irrelevant events caused %d snapshot writes", kv.writes)
	}
	for _, evt := range []*event.Event{
		{Type: event.StateMember, StateKey: &user, Content: event.Content{Parsed: &event.MemberEventContent{Membership: event.MembershipJoin}}},
		{Type: event.StatePowerLevels, StateKey: &empty, Content: event.Content{Parsed: &event.PowerLevelsEventContent{UsersDefault: 42}}},
		{Type: event.StateEncryption, StateKey: &empty, Content: event.Content{Parsed: &event.EncryptionEventContent{Algorithm: id.AlgorithmMegolmV1}}},
		{Type: event.StateCreate, StateKey: &empty, Content: event.Content{Parsed: &event.CreateEventContent{}}},
		{Type: event.StateJoinRules, StateKey: &empty, Content: event.Content{Parsed: &event.JoinRulesEventContent{JoinRule: event.JoinRuleInvite}}},
	} {
		evt.RoomID = "!room:example"
		store.UpdateState(ctx, evt)
	}
	if kv.writes != 5 {
		t.Fatalf("expected one write per stored state event, got %d", kv.writes)
	}
	restored, err := newPersistentStateStore(ctx, kv, "state")
	if err != nil {
		t.Fatal(err)
	}
	if restored.Members["!room:example"][id.UserID(user)].Membership != event.MembershipJoin || restored.PowerLevels["!room:example"].UsersDefault != 42 || restored.Encryption["!room:example"].Algorithm != id.AlgorithmMegolmV1 || restored.Create["!room:example"] == nil || restored.JoinRules["!room:example"].JoinRule != event.JoinRuleInvite {
		t.Fatal("state lost across restart")
	}
}
