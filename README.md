# Imap component

This uses DKIM verification from our fork of Cloudflare's DKIM library: https://github.com/Lay3rLabs/dkim-wasi

## Getting Started

1. Set your credentials

```bash
cp .env.example .env
# Then edit the .env file to set your IMAP credentials
# See below about gmail
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

## Gmail setup

### Prerequisites

Aside for the typical Taskfile, Rust, jq, etc., you'll need a local HTTP server to handle the OAuth redirect.
Any http server under the `http-server` command that can serve the local directory and take `-p <port>` as an argument is fine
For example, http-server from cargo:

```bash
cargo install http-server
```

### 1. Create OAuth Client in Google Cloud Console

1. Go to [Google Cloud Console → APIs & Services → Credentials](https://console.cloud.google.com/apis/credentials)
2. Click "Create Credentials" → "OAuth client ID"
3. Choose **"Desktop app"** as the application type
4. Save the Client ID and Client Secret

Note: The OAuth task automatically finds a free port starting at 53682. In testing mode, Google may allow localhost redirects without explicit URI configuration.

### 2. Configure OAuth Consent Screen

1. Go to OAuth consent screen in Google Cloud Console
2. Add required scopes:
   - `https://mail.google.com/` (full Gmail access for IMAP)
3. Add test users if in "Testing" mode

### 3. Generate OAuth Tokens

Set your credentials in `.env`:
```bash
WAVS_ENV_GMAIL_CLIENT_ID=your-client-id
WAVS_ENV_GMAIL_CLIENT_SECRET=your-client-secret
```

Then run the bootstrap task:
```bash
task gmail-bootstrap
```

This will:
1. Auto-find a free port starting at 53682 and start a local HTTP server
2. Generate a secure OAuth URL with PKCE
3. Display the URL for you to open in your browser
4. Show the auth code in a beautiful web page after you authorize
5. Prompt you to paste the code back into the terminal
6. Exchange it for a refresh token
7. Automatically stop the local server

Add the output to your `.env` file.

### Security Notes

- The OAuth flow uses **PKCE** (Proof Key for Code Exchange) for added security
- Client secrets for Desktop apps are [not really secret](https://developers.google.com/identity/protocols/oauth2/native-app):
  > "Installed apps are distributed to individual devices, and it is assumed that these apps cannot keep secrets"
- PKCE prevents authorization code interception attacks
- The local server runs only during the OAuth flow and is automatically shut down

### Production Use

For production, you'll need to go through Google's client verification process to remove the "unverified app" warning.

## Notes

wasi:tls is not on wa.dev, so we just clone our wit repo
