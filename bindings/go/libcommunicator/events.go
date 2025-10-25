package libcommunicator

import (
	"context"
	"sync"
	"time"
)

// EventStream provides a Go-idiomatic way to consume platform events
type EventStream struct {
	platform *Platform
	events   chan *Event
	errors   chan error
	done     chan struct{}
	wg       sync.WaitGroup
	once     sync.Once
}

// NewEventStream creates a new event stream for the platform
// The stream will poll for events in the background and send them to a channel
func (p *Platform) NewEventStream(ctx context.Context, bufferSize int) (*EventStream, error) {
	if err := p.SubscribeEvents(); err != nil {
		return nil, err
	}

	stream := &EventStream{
		platform: p,
		events:   make(chan *Event, bufferSize),
		errors:   make(chan error, 10),
		done:     make(chan struct{}),
	}

	stream.wg.Add(1)
	go stream.poll(ctx)

	return stream, nil
}

// Events returns a read-only channel for receiving events
func (s *EventStream) Events() <-chan *Event {
	return s.events
}

// Errors returns a read-only channel for receiving errors
func (s *EventStream) Errors() <-chan error {
	return s.errors
}

// poll continuously polls for events in the background
func (s *EventStream) poll(ctx context.Context) {
	defer s.wg.Done()
	defer close(s.events)
	defer close(s.errors)

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-s.done:
			return
		case <-ticker.C:
			// Poll for events
			event, err := s.platform.PollEvent()
			if err != nil {
				select {
				case s.errors <- err:
				default:
					// Error channel is full, drop the error
				}
				continue
			}

			if event != nil {
				select {
				case s.events <- event:
				case <-ctx.Done():
					return
				case <-s.done:
					return
				}
			}
		}
	}
}

// Close closes the event stream
func (s *EventStream) Close() error {
	var err error
	s.once.Do(func() {
		close(s.done)
		s.wg.Wait()
		err = s.platform.UnsubscribeEvents()
	})
	return err
}

// EventHandler is a function that handles events
type EventHandler func(*Event)

// EventRouter routes events to handlers based on event type
type EventRouter struct {
	handlers map[string][]EventHandler
	mu       sync.RWMutex
}

// NewEventRouter creates a new event router
func NewEventRouter() *EventRouter {
	return &EventRouter{
		handlers: make(map[string][]EventHandler),
	}
}

// On registers a handler for a specific event type
func (r *EventRouter) On(eventType string, handler EventHandler) {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.handlers[eventType] = append(r.handlers[eventType], handler)
}

// OnMessagePosted registers a handler for message posted events
func (r *EventRouter) OnMessagePosted(handler EventHandler) {
	r.On(EventMessagePosted, handler)
}

// OnMessageUpdated registers a handler for message updated events
func (r *EventRouter) OnMessageUpdated(handler EventHandler) {
	r.On(EventMessageUpdated, handler)
}

// OnMessageDeleted registers a handler for message deleted events
func (r *EventRouter) OnMessageDeleted(handler EventHandler) {
	r.On(EventMessageDeleted, handler)
}

// OnUserStatusChanged registers a handler for user status changed events
func (r *EventRouter) OnUserStatusChanged(handler EventHandler) {
	r.On(EventUserStatusChanged, handler)
}

// OnUserTyping registers a handler for user typing events
func (r *EventRouter) OnUserTyping(handler EventHandler) {
	r.On(EventUserTyping, handler)
}

// OnChannelCreated registers a handler for channel created events
func (r *EventRouter) OnChannelCreated(handler EventHandler) {
	r.On(EventChannelCreated, handler)
}

// OnChannelUpdated registers a handler for channel updated events
func (r *EventRouter) OnChannelUpdated(handler EventHandler) {
	r.On(EventChannelUpdated, handler)
}

// OnChannelDeleted registers a handler for channel deleted events
func (r *EventRouter) OnChannelDeleted(handler EventHandler) {
	r.On(EventChannelDeleted, handler)
}

// OnUserJoinedChannel registers a handler for user joined channel events
func (r *EventRouter) OnUserJoinedChannel(handler EventHandler) {
	r.On(EventUserJoinedChannel, handler)
}

// OnUserLeftChannel registers a handler for user left channel events
func (r *EventRouter) OnUserLeftChannel(handler EventHandler) {
	r.On(EventUserLeftChannel, handler)
}

// OnConnectionStateChanged registers a handler for connection state changed events
func (r *EventRouter) OnConnectionStateChanged(handler EventHandler) {
	r.On(EventConnectionStateChange, handler)
}

// Handle dispatches an event to all registered handlers
func (r *EventRouter) Handle(event *Event) {
	r.mu.RLock()
	handlers, ok := r.handlers[event.Type]
	r.mu.RUnlock()

	if !ok {
		return
	}

	for _, handler := range handlers {
		handler(event)
	}
}

// Run starts the event router with an event stream
// It will block until the context is cancelled
func (r *EventRouter) Run(ctx context.Context, stream *EventStream) error {
	defer stream.Close()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case event, ok := <-stream.Events():
			if !ok {
				return nil
			}
			r.Handle(event)
		case err, ok := <-stream.Errors():
			if !ok {
				return nil
			}
			// Log or handle errors appropriately
			// For now, we just continue
			_ = err
		}
	}
}
