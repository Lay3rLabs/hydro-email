We've already setup a client, so all you really need to do is:

```bash
task oauth:gmail-bootstrap
```
And follow the instructions to ultimately get your `WAVS_ENV_GMAIL_TOKEN` value to add to your `.env` file.

However, if you need to setup a new OAuth client from scratch, follow these steps:

### 1. Create OAuth Client in Google Cloud Console

1. Go to [Google Cloud Console → APIs & Services → Credentials](https://console.cloud.google.com/apis/credentials)
2. Click "Create Credentials" → "OAuth client ID"
3. Choose **"Desktop app"** as the application type
4. Save the Client ID and Client Secret

Note: The `oauth:gmail-bootstrap` task automatically finds a free port starting at 53682.

### 2. Configure OAuth Consent Screen

1. Go to OAuth consent screen in Google Cloud Console
2. Add required scopes:
   - `https://mail.google.com/` (full Gmail access for IMAP)
   - `https://www.googleapis.com/auth/gmail.readonly` (for reading emails through REST API)
   - `https://www.googleapis.com/auth/gmail.modify` (for modifying read state through REST API)
3. Add test users if in "Testing" mode

### 3. Generate OAuth Tokens

Set your credentials in `.env`:

```bash
WAVS_ENV_GMAIL_CLIENT_ID=your-client-id
WAVS_ENV_GMAIL_CLIENT_SECRET=your-client-secret
```

Then run the bootstrap task as usual:
```bash
task oauth:gmail-bootstrap
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
