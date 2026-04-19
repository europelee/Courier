## ADDED Requirements

### Requirement: Client registers tunnel over WebSocket
courier-client SHALL send a `register` JSON message as the first frame upon connecting to `/ws`. The server SHALL respond with a `tunnel_established` JSON message containing `courier_id`, `subdomain`, `public_url`, and `server_domain`.

#### Scenario: Successful tunnel registration
- **WHEN** courier-client connects to `/ws` and sends `{"msg_type":"register","data":{...}}`
- **THEN** server responds with `{"msg_type":"tunnel_established","data":{"courier_id":"...","subdomain":"...","public_url":"...","server_domain":"..."}}`

#### Scenario: Registration with empty auth token
- **WHEN** courier-client sends a `register` message with empty `auth_token`
- **THEN** server closes the WebSocket connection with an error frame

### Requirement: Public traffic forwarded via Binary frames
After registration, the server SHALL forward incoming public HTTP requests to the connected courier-client as WebSocket Binary frames. The courier-client SHALL proxy the request to the local service and send the response back as a Binary frame. The server SHALL relay the response to the original requester.

#### Scenario: Public request forwarded to client
- **WHEN** a public HTTP request arrives for a registered subdomain
- **THEN** server sends the raw request bytes as a Binary WebSocket frame to the corresponding courier-client

#### Scenario: Client sends response back
- **WHEN** courier-client sends a Binary frame containing the HTTP response bytes
- **THEN** server relays those bytes as the HTTP response to the original requester

### Requirement: Heartbeat keeps connection alive
The server SHALL respond to `{"msg_type":"heartbeat"}` JSON messages with `{"msg_type":"heartbeat_ack"}`.

#### Scenario: Heartbeat acknowledged
- **WHEN** courier-client sends `{"msg_type":"heartbeat","data":{"courier_id":"...","timestamp":...}}`
- **THEN** server responds with `{"msg_type":"heartbeat_ack","data":{"status":"ok"}}`

### Requirement: Tunnel removed on disconnect
When a courier-client WebSocket connection closes, the server SHALL remove the tunnel from `TunnelRegistry` and mark it as disconnected in the database.

#### Scenario: Client disconnects
- **WHEN** courier-client closes the WebSocket connection
- **THEN** the tunnel entry is removed from `TunnelRegistry` and its status set to `disconnected` in the database
