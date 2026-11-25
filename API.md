# API

The HTTP API listens on `0.0.0.0:42069`. Use `http://localhost:42069` when the server is running locally.

The API is read/write but is not behind authentication. JSON request and response bodies use `application/json` unless noted otherwise.

## Health

### `GET /health`

Checks whether the API server is reachable.

**Response**

- `200 OK` with an empty body

## Commands

### `GET /commands`

Returns all custom commands.

**Response**

- `200 OK`

```json
[
  {
    "name": "discord",
    "alias": "dc",
    "response": "Join us at https://example.com/discord"
  }
]
```

`alias` is `null` when the command has no alias.

### `POST /command/create`

Creates a custom command.

**Request**

```json
{
  "name": "discord",
  "alias": "dc",
  "response": "Join us at https://example.com/discord"
}
```

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | yes | Command name. |
| `alias` | string or null | no | Optional command alias. |
| `response` | string | yes | Text returned for the command. |

**Response**

- `201 Created` when the command is stored
- `500 Internal Server Error` when it cannot be stored

The success response has an empty body.

Example:

```bash
curl -i -X POST http://localhost:42069/command/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "discord",
    "alias": "dc",
    "response": "Join us at https://example.com/discord"
  }'
```

### `DELETE /command/{name}`

Removes a command by its exact name.

**Path parameters**

| Parameter | Type | Description |
| --- | --- | --- |
| `name` | string | Exact command name to remove. |

**Response**

- `204 No Content` when the command is removed
- `500 Internal Server Error` when the database operation fails

The success response has an empty body.

Example:

```bash
curl -i -X DELETE http://localhost:42069/command/discord
```

## Widgets

Widgets are uploaded as ZIP archives. The archive must contain a top-level `index.html` file. The widget name is derived from the uploaded ZIP filename without its extension.

### `GET /widgets`

Lists every uploaded widget.

**Response**

- `200 OK`

```json
[
  {
    "id": 1,
    "name": "overlay",
    "created_at": "2026-11-25 15:02:15"
  }
]
```

- `500 Internal Server Error` when widgets cannot be read

### `POST /widget/upload`

Uploads a widget ZIP archive.

**Request**

Send the ZIP archive as `multipart/form-data`, using any field name:

```bash
curl -i -X POST http://localhost:42069/widget/upload \
  -F "widget=@overlay.zip"
```

The API inspects only the first uploaded field and requires a filename from which to derive the widget name.

**Response**

- `200 OK`

```json
{
  "id": "1",
  "name": "overlay"
}
```

`id` is returned as a string in this endpoint.

- `400 Bad Request` when the multipart field is missing or malformed, the ZIP is invalid, no top-level `index.html` exists, or extraction fails
- `500 Internal Server Error` when the widget database record cannot be created

### `GET /widget/{id}`

Redirects to the widget's trailing-slash URL.

**Response**

- `308 Permanent Redirect`, with `Location: /widget/{id}/`

### `GET /widget/{id}/`

Returns the widget's `index.html` as HTML.

**Path parameters**

| Parameter | Type | Description |
| --- | --- | --- |
| `id` | integer | Widget database ID. |

**Response**

- `200 OK` with `text/html` semantics
- `400 Bad Request` when the widget ID is unknown or the extracted widget files are missing
- `500 Internal Server Error` when widget database lookup fails

The server reads `./data/widgets/{id}/index.html`.

Example:

```text
http://localhost:42069/widget/1/
```

### `GET /widget/{id}/{*asset}`

Returns a file belonging to a widget.

**Path parameters**

| Parameter | Type | Description |
| --- | --- | --- |
| `id` | integer | Widget database ID. |
| `asset` | string | Relative file path below the widget directory. It may contain nested path segments. |

The response `Content-Type` is inferred from the file extension, or falls back to `application/octet-stream`.

**Response**

- `200 OK` with the file contents
- `400 Bad Request` when the widget ID is unknown
- `404 Not Found` when the widget directory or asset does not exist or the asset is not a file
- `500 Internal Server Error` for database lookup or file read failures

Example:

```text
http://localhost:42069/widget/1/styles.css
http://localhost:42069/widget/1/assets/logo.png
```

### `DELETE /widget/{id}`

Deletes a widget database record and removes its extracted files.

**Response**

- `200 OK` with an empty body when the widget is deleted
- `400 Bad Request` when the widget ID is unknown or the widget directory does not exist
- `500 Internal Server Error` when the database record or extracted directory cannot be removed


## WebSocket events

### `GET /ws`

Upgrades the connection to a WebSocket and subscribes the client to bot events. Event data is sent as JSON text messages.

The broadcast subscription starts when the WebSocket is opened. Events received before the connection opens are not replayed.

**Message event**

```json
{
  "type": "message",
  "data": {
    "user_id": "123456",
    "username": "viewer",
    "content": "Hello"
  }
}
```

**Raid event**

```json
{
  "type": "raid",
  "data": {
    "user_id": "123456",
    "username": "raider",
    "viewers": 42
  }
}
```

Example:

```bash
curl --include --http1.1 \
  --header "Connection: Upgrade" \
  --header "Upgrade: websocket" \
  --header "Sec-WebSocket-Version: 13" \
  --header "Sec-WebSocket-Key: $(openssl rand -base64 16)" \
  http://localhost:42069/ws
```

Use a WebSocket client for normal use; `wscat -c ws://localhost:42069/ws` is an example.
