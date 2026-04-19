## ADDED Requirements

### Requirement: Frontend subscribes to tunnel events via WebSocket
The web frontend SHALL connect to `/ws` and send `{"msg_type":"subscribe","data":{}}` as the first frame. The server SHALL register this connection as a subscriber in `TunnelRegistry`.

#### Scenario: Successful subscription
- **WHEN** frontend connects to `/ws` and sends `{"msg_type":"subscribe","data":{}}`
- **THEN** server registers the connection as a subscriber and immediately pushes a snapshot of all currently active tunnels

### Requirement: Server pushes tunnel_connected event
When a courier-client successfully registers a tunnel, the server SHALL broadcast a `tunnel_connected` event to all frontend subscribers.

#### Scenario: New tunnel comes online
- **WHEN** a courier-client completes tunnel registration
- **THEN** server broadcasts `{"msg_type":"tunnel_connected","data":{"courier_id":"...","subdomain":"...","public_url":"...","local_port":...}}` to all subscribers

### Requirement: Server pushes tunnel_disconnected event
When a courier-client disconnects, the server SHALL broadcast a `tunnel_disconnected` event to all frontend subscribers.

#### Scenario: Tunnel goes offline
- **WHEN** a courier-client WebSocket connection closes
- **THEN** server broadcasts `{"msg_type":"tunnel_disconnected","data":{"courier_id":"..."}}` to all subscribers

### Requirement: Server pushes stats_update event
The server SHALL broadcast a `stats_update` event to all frontend subscribers every 10 seconds, containing `bytes_transferred` for each active tunnel.

#### Scenario: Periodic stats pushed to frontend
- **WHEN** 10 seconds have elapsed since the last stats broadcast
- **THEN** server broadcasts `{"msg_type":"stats_update","data":{"tunnels":[{"courier_id":"...","bytes_transferred":...}]}}` to all subscribers

### Requirement: Frontend replaces polling with WebSocket
The frontend SHALL NOT use `setInterval` to poll REST endpoints for tunnel status. All real-time state updates SHALL come from the WebSocket subscription.

#### Scenario: Tunnel list updates without polling
- **WHEN** a `tunnel_connected` or `tunnel_disconnected` event is received
- **THEN** the frontend tunnel list updates immediately without any REST API call
