# Imap component

## Getting Started

1. Set your credentials

```bash
cp .env.example .env
# Then edit the .env file to set your IMAP credentials
```

2. Pull the docker images

```bash
task docker-pull
```

3. Fetch the wit

```bash
task fetch-wit
```

4. Build the component

```bash
task build
```

5. (optional, local testing) Start the local IMAP server

```bash
task mail-server-start
```

6. Execute the component to check for latest mail

```bash
task exec
```

5. (optional, local testing) Send an email (then check again!)

```bash
task mail-server-send
```

7. (optional, local testing) Stop the local IMAP server

```bash
task mail-server-stop
```

## Development

To rebuild and re-execute on file changes, run:

```bash
task watch
```

## Notes

wasi:tls is not on wa.dev, so we just clone our wit repo
