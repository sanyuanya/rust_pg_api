# rust_pg_api

A tiny Rust + Postgres API service for learning database access.

## Local Setup

1) Start Postgres and create a database:

```sql
CREATE DATABASE rust_pg_api;
```

2) Create the table:

```bash
psql postgres://postgres:postgres@localhost:5432/rust_pg_api -f db/init.sql
```

3) Configure env:

```bash
cp .env.example .env
```

4) Run the server:

```bash
cargo run
```

Server runs on `http://127.0.0.1:3000`.

## Docker

From `rust_pg_api/`:

```bash
docker compose up --build
```

- API: `http://127.0.0.1:3000`
- Postgres: `localhost:5432`
- Default user/pass: `postgres/postgres`

To stop:

```bash
docker compose down
```

To remove DB data:

```bash
docker compose down -v
```

## Endpoints

- `GET /health`
- `GET /todos`
- `POST /todos` with JSON `{ "title": "learn sqlx" }`
- `GET /todos/:id`
- `PATCH /todos/:id` with JSON `{ "completed": true }`
- `DELETE /todos/:id`

## Quick test

```bash
curl -s http://127.0.0.1:3000/health

curl -s -X POST http://127.0.0.1:3000/todos \
  -H 'content-type: application/json' \
  -d '{"title":"learn rust sqlx"}'

curl -s http://127.0.0.1:3000/todos
```
