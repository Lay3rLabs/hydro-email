## Set environment

In the `.env`, make sure to set `DEPLOY_*` as needed

## Upload and deploy just the artifacts

```
task deploy:artifacts
task deploy:service
```

## For each user (change `DEPLOY_REGISTER_USER_EMAIL` in the `.env`):

```
task deploy:user
```

## Make sure each backend service is running (per-node)

```
task backend:start-wavs
```

## Register it for each operator (per-operator)

Easiest is just to set `WAVS_OPERATOR_SIGNING_MNEMONIC_1` and `REMOTE_WAVS_OPERATOR_URL_1` and call this remotely, replacing the env vars for each operator:

```
task deploy:operator
```
