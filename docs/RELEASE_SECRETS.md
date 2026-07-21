# Release signing and secrets

GitHub Actions builds unsigned review artifacts on ordinary branches when secrets are unavailable. A `v*` tag is stricter: the macOS app and plug-in workflows refuse to publish unless their signing/notarization requirements are present.

Never print secret values. To audit names only:

```bash
gh secret list --json name --jq '.[].name' | sort
```

## macOS app and plug-ins

Required for a tagged release:

- `APPLE_CERTIFICATE_BASE64` — base64 `.p12` containing the Developer ID Application certificate
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY` — for example `Developer ID Application: Contrapunk Audio (TEAMID)`
- `APPLE_TEAM_ID`
- `APPLE_API_KEY_BASE64` — base64 App Store Connect `AuthKey_*.p8`
- `APPLE_API_KEY` — App Store Connect key ID
- `APPLE_API_ISSUER` — App Store Connect issuer ID
- `APPLE_INSTALLER_SIGNING_IDENTITY` — for example `Developer ID Installer: Contrapunk Audio (TEAMID)`

If the installer certificate is not included in `APPLE_CERTIFICATE_BASE64`, also set:

- `APPLE_INSTALLER_CERTIFICATE_BASE64`
- `APPLE_INSTALLER_CERTIFICATE_PASSWORD`

Example setup:

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

Tagged workflows verify the `.app`, notarize and staple the DMG, sign the VST3/CLAP/AU bundles, sign the plug-in PKG, and notarize/staple that PKG. Do not describe a local or branch artifact as signed/notarized unless the artifact itself passes verification.

### Verify downloaded macOS candidates

```bash
codesign --verify --deep --strict --verbose=2 /path/to/Contrapunk.app
spctl --assess --type execute --verbose /path/to/Contrapunk.app
xcrun stapler validate /path/to/Contrapunk_*.dmg

pkgutil --check-signature /path/to/Contrapunk-Plugins-macOS-universal.pkg
xcrun stapler validate /path/to/Contrapunk-Plugins-macOS-universal.pkg
```

For raw plug-in bundles, verify each extracted `.vst3`, `.clap`, and `.component` with `codesign --verify --deep --strict`.

## Windows

The Windows desktop workflow currently builds NSIS/MSI artifacts without an Authenticode signing step.

The Windows plug-in workflow can sign its NSIS installer when these optional secrets are present:

- `WINDOWS_CERTIFICATE_BASE64`
- `WINDOWS_CERTIFICATE_PASSWORD`
- `WINDOWS_CERTIFICATE_THUMBPRINT`

If they are absent, the workflow still produces the plug-in installer and ZIP, but they are **unsigned**. Raw VST3/CLAP bundles are not signed by the current workflow. Release notes and download pages must say this plainly.

Verify an Authenticode claim on Windows with:

```powershell
Get-AuthenticodeSignature .\Contrapunk-Plugins-Windows-x64.exe | Format-List
```

Only `Status: Valid` supports calling the installer signed.

## Pre-tag gate

Before creating `v1.3.0`:

1. Confirm all required Apple secret names exist.
2. Confirm whether the three Windows signing secret names exist; record unsigned status if not.
3. Build and download all six release candidates from the exact release commit.
4. Verify architectures, signatures/notarization where applicable, archive contents, checksums, and smoke tests.
5. Present the release-candidate manifest and obtain explicit human approval.
6. Only then create and push the tag. Never replace or force-update a public tag.
