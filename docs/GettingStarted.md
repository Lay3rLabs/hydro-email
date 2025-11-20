## Prerequisites

1. The usual stuff (Rust, Docker, NPM, etc.)
2. [Taskfile](https://taskfile.dev/installation)
3. [Install and configure wkg to pull from wa.dev](https://crates.io/crates/wkg)
4. Make sure you've pulled all the docker images: `task docker-pull`
5. Install any `http-server` command that takes `-p <port>` as an argument (for example, `http-server` from cargo)
6. Make sure you have `wasm32-wasip2` target installed: `rustup target add wasm32-wasip2`
7. Build the helper binary: `task build-helper` (this will take a while for the first time)
8. Download the latest WIT definitions: `task components:fetch-wit`
9. Copy `.example.env` to `.env` and replace the values

## Summary

Start all backend services
```bash
# Alternatively, if you need more than one operator
# task backend:start-all OPERATORS=3
task backend:start-all
```

_... do stuff ..._

Stop all backend services
```bash
task backend:stop-all
```

But "do stuff" assumes you've build and tested all the contracts and components
and sometimes you only want to start/stop parts of the backend

So, with that in mind...

## Building

### Contracts

To build the contracts (found in `.builds/contracts/`):

```bash
task contracts:build-all
```

To generate the schema files from the contracts (found in `.builds/schema/`)

```bash
task contracts:schema-all
```


### Components

#### First, fetch the wit definitions

This is only needed once, or when the component wits are updated, and you probably already did that in the prerequisites

```bash
task components:fetch-wit
```

#### Build the components

```bash
task components:build-all
```

#### Execute a component directly

This will execute the operator component to read latest emails 

```bash
task components:exec-operator-read-mail
```

## Backend

### Chains

It may take a while for the chain to startup, be patient... chains will be running in the background via docker and do not require their own terminal

Start the chains
```bash
task backend:start-chains
```

Stop the chains
```bash
task backend:stop-chains
```

### WAVS

Start the operator, aggregator, and telemetry
```bash
# Alternatively, if you need more than one operator
# task backend:start-wavs OPERATORS=3
task backend:start-wavs
```

Stop the operator, aggregator, and telemetry
```bash
task backend:stop-wavs
```

### IPFS

Start a local IPFS server

```bash
task backend:start-ipfs
```

Stop the local IPFS server
```bash
task backend:stop-ipfs
```

### All Backend Services At Once

Start all backend services
```bash
# Alternatively, if you need more than one operator
# task backend:start-all OPERATORS=3
task backend:start-all
```

Stop all backend services
```bash
task backend:stop-all
```

## Testing


### Contracts

All off-chain tests
```bash
task test:off-chain
```

*on-chain*

Make sure you've [started the chains](#chains) first

```bash
task test:on-chain
```

### Components

Just `cargo test` as needed

### Telemetry UIs

Jaeger UI is at [http://localhost:16686/](http://localhost:16686/)
Prometheus is at [http://localhost:9092/](http://localhost:9092/)
