# MVP OpenAPI Rust Server / Typescript Client Example

## What

This example is an MVP to run a basic typesafe OpenAPI Server in Rust and an OpenAPI Client in Typescript. This example:

### Server

- Uses https://github.com/oxidecomputer/dropshot to generate an OpenAPI specification document which is rebuilt automatically as source file changes. See `server/api/v1.json` produced by `server/build.rs`.
- Demonstrates how to implement an OpenAPI endpoint in `server/src/routes/counter.rs`.
- Uses https://github.com/oxidecomputer/progenitor to generate an OpenAPI client from that specification for testing. See `mod test` in `server/src/routes/counter.rs`.
- Uses https://github.com/rusqlite/rusqlite.git to persist the data to a SQLite instance including tests. See `server/src/entity/counter.rs`.
- Demonstrates how to write/read from the database in `server/src/routes/counter.rs`.

### Client

- Uses https://github.com/drwpow/openapi-typescript/tree/main/packages/openapi-typescript to consume the `server/api/v1.json` and produce the typescript definitions to `client/src/api/v1.d.ts`.
- Uses https://github.com/drwpow/openapi-typescript/tree/main/packages/openapi-fetch to interact with the server via the `client/src/index.ts` example.

## Deploy

This example was built with the intention of using [Litestream](https://litestream.io/) to provide a continuous replication to a storage bucket (s3 etc.). Using Litestream's build-in process supervision a single container can be built that bundles both the application and Litestream together (https://litestream.io/guides/docker/#running-in-the-same-container).

See this repository from the Litestream authors on how this works: https://github.com/benbjohnson/litestream-docker-example/.

To build this image:

```bash
docker build . -f deploy/Dockerfile -t server:latest
```

## Warnings

SQLite does have some drawbacks compared to a database such as Postgres in that only a single writer can write at a time. This repository does use the Write-Ahead-Log mode (https://www.sqlite.org/wal.html) but care must be taken:

- Do not hold a Database transaction open while `.await`ing a long task. For example, do not make a request to an external API within a `database.write()` transaction as it will block all writes.
- If writes are not needed use a `database.read()` transaction which runs in a pool of read-only connections to SQLite and, due to the WAL, will run concurrently.