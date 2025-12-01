# Prerequisites

Before getting started, make sure to install all the necessary [Tooling](./Tooling.md)

# First time

1. Pull all the docker images: `task docker-pull`
2. Build the helper binary: `task build-helper` (this will take a while for the first time)
3. Download the latest WIT definitions: `task components:fetch-wit`
4. Copy `.example.env` to `.env` and replace the values

## TL;DR

1. Build all contracts, components, and schema

```bash
task build-all
```

2. Start all backend services

```bash
task backend:start-all

# Alternatively, if you need more than one operator
# 
# task backend:start-all OPERATORS=3
```

3. Tap the faucet for all the mnemonics

```bash
task deploy:tap-faucet-all
```

4. Deploy everything

```bash
task deploy:all

# Alternatively, skip uploading contracts, middleware, and/or components
# this assumes they were already uploaded before and the output files exist
# 
# task deploy:all SKIP_UPLOAD_CONTRACTS=true SKIP_UPLOAD_COMPONENTS=true SKIP_UPLOAD_MIDDLEWARE=true
```

5. Send an email to the operator email address

See [LocalEmail](./LocalEmail.md) for using the local email server for testing, otherwise, use a real email server

6. Check the on-chain results

This will use the same `DEPLOY_CHAIN_TARGET` env var as the deploy commands to determine which chain to query

```bash
task contracts:query-service-handler-emails
task contracts:query-proxy-state
```

7. When done, stop all backend services

```bash
task backend:stop-all
```

Each of these steps can be done individually and with more granularity as needed.

Let's step through each of these:



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
task components:exec-read-mail
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
