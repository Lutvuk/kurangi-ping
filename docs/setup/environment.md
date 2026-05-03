# Environment Setup

This document defines the required environment variables for Kurangi Ping 2.

## Required Variables

| Variable | Required | Example | Notes |
|---|---|---|---|
| `KP_ENV` | Yes | `dev` | Allowed baseline: `dev`, `staging`, `prod`. |
| `KP_MANIFEST_URL` | Yes | `https://example.com/v1/relay/manifest` | HTTPS endpoint for signed relay manifest. |
| `KP_MANIFEST_PUBKEY` | Yes | `BASE64_PUBLIC_KEY` | Public key used to verify manifest signature. |
| `KP_TELEMETRY_ENDPOINT` | Yes | `https://example.com/v1/telemetry/events:batch` | Telemetry batch ingest endpoint (HTTPS only). |
| `KP_UPDATE_CHANNEL` | Yes | `beta` | Allowed baseline: `beta`, `stable`. |
| `KP_RELAY_HEALTH_URL` | Yes | `https://example.com/v1/relay/health` | Relay health summary endpoint. |

## Safe Local Setup

1. Copy `.env.example` to `.env.local`.
2. Fill values with local or staging-safe endpoints.
3. Never commit `.env.local` or any secret-bearing env file.
4. Use placeholder values for documentation and samples.
5. Do not store private keys in repository files.

PowerShell example:

```powershell
Copy-Item .env.example .env.local
```

## Security Boundaries

- `.env.example` may be committed because it contains placeholders only.
- Secret-bearing env files are ignored by `.gitignore`.
- Private key material is prohibited in source control.
