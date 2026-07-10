# Release secrets

Set these GitHub Actions secrets before pushing a `v*` tag.

## macOS app + plugins

Required:

- `APPLE_CERTIFICATE_BASE64` — base64 `.p12` containing Developer ID Application cert.
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY` — e.g. `Developer ID Application: Contrapunk Audio (TEAMID)`.
- `APPLE_TEAM_ID`
- `APPLE_API_KEY_BASE64` — base64 App Store Connect `AuthKey_*.p8`.
- `APPLE_API_KEY` — App Store Connect key ID.
- `APPLE_API_ISSUER` — App Store Connect issuer ID.
- `APPLE_INSTALLER_SIGNING_IDENTITY` — e.g. `Developer ID Installer: Contrapunk Audio (TEAMID)`.

Optional if the installer cert is not already inside `APPLE_CERTIFICATE_BASE64`:

- `APPLE_INSTALLER_CERTIFICATE_BASE64`
- `APPLE_INSTALLER_CERTIFICATE_PASSWORD`

Commands:

```bash
base64 -i DeveloperIDApplication.p12 | gh secret set APPLE_CERTIFICATE_BASE64
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set APPLE_SIGNING_IDENTITY
gh secret set APPLE_TEAM_ID

base64 -i DeveloperIDInstaller.p12 | gh secret set APPLE_INSTALLER_CERTIFICATE_BASE64
gh secret set APPLE_INSTALLER_CERTIFICATE_PASSWORD
gh secret set APPLE_INSTALLER_SIGNING_IDENTITY

base64 -i AuthKey_XXXXXXXXXX.p8 | gh secret set APPLE_API_KEY_BASE64
gh secret set APPLE_API_KEY
gh secret set APPLE_API_ISSUER
```

## Windows plugin installer signing

Optional for now:

- `WINDOWS_CERTIFICATE_BASE64`
- `WINDOWS_CERTIFICATE_PASSWORD`
- `WINDOWS_CERTIFICATE_THUMBPRINT`

The Windows plugin workflow still produces an installer without these; when present, it Authenticode-signs the `.exe`.
