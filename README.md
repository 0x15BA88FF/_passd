# PASSD – Password Store Daemon

## Overview

**PASSD** is a secure, file-based password manager implemented in Rust. It runs
as a local or remote daemon over JSON-RPC via HTTP(S), and is designed for
**single-user** usage.

---

## Configuration

Configuration is loaded from the system’s standard config directories or a
custom path, supporting:

* `$XDG_CONFIG_HOME` or `~/.config/` (Unix)
* `C:\Users\{User}\AppData\Local\` (Windows)
* The environment variable `PASSD_CONFIG_DIR`, if set

### Config File Priority

1. `${PASSD_CONFIG_DIR}/config.toml`
2. `~/.config/passd/config.toml`
3. `~/.passd/config.toml`

### Config Format (TOML)

```toml
# Absolute path to the vault directory
vault_dir = "~/.local/share/passd/"

# Absolute path where logs are stored
logs_dir = "~/.local/state/passd.log"

# Absolute path where metadata is stored
metadata_dir = "metadata/"

# Local port to run the JSON-RPC server
port = 8080

# Enable HTTPS/TLS support
enable_tls = true

public_key_path = "/home/user/.keys/passd.pub"
private_key_path = "/home/user/.keys/passd.sec"
```

---

## Vault Structure

The vault is a file-based directory containing all secrets and metadata. Only
two file types are valid:

* Encrypted secret files (`.pgp`)
* Corresponding unencrypted metadata files (`.meta.toml`)

> ❌ Any other file types are rejected.

### Permissions

* Vault directories: `700`
* Vault files: `600`

### File Naming Convention

Every secret must have a matching metadata file:

| Secret File          | Metadata File              |
| -------------------- | -------------------------- |
| `my-password.pgp`    | `my-password.meta.toml`    |
| `my-token.asc.pgp`   | `my-token.asc.meta.toml`   |
| `my-image.png.pgp`   | `my-image.png.meta.toml`   |
| `some.meta.toml.pgp` | `some.meta.toml.meta.toml` |

---

## Sidecar Metadata (`.meta.toml`)

Each metadata file contains unencrypted attributes describing its paired secret.

### Metadata Template

This is the default template used when generating new metadata:

```toml
[metadata_template]
type = "Untitled Secret"
category = "default"
tags = ["uncategorized"]
description = "No description provided"
attachments = []
```

Users can customize this in their config and add additional fields.

### Auto-managed Fields

PASSD automatically sets and updates these:

```toml
modifications = 1                  # Increments on every change
fingerprint = "c345...abcd"       # PGP fingerprint that encrypted the secret

created_at = "2025-07-12T10:00:00Z"
updated_at = "2025-07-13T10:00:00Z"

checksum_main = "c345...abcd"     # SHA-256 of the encrypted secret
checksum_meta = "d123...ef56"     # SHA-256 of this metadata file
```

### Metadata Example

```toml
name = "My SSH Key"
type = "token"
category = "work"
tags = ["ssh", "prod"]

modifications = 1
fingerprint = "c345...abcd"

created_at = "2025-07-12T10:00:00Z"
updated_at = "2025-07-13T10:00:00Z"

checksum_main = "c345...abcd"
checksum_meta = "d123...ef56"
```

> Secrets without valid metadata are ignored by operations like `find`,
`read`, `edit`, etc.

### Metadata Errors

* Invalid or missing `.meta.toml` causes the operation to **fail**
* Commands are available to diagnose and regenerate broken metadata files

---

## Server Architecture

PASSD follows a **controller-based architecture** with these key properties:

* All operations are **queued** to guarantee safe single-user concurrency
* Sensitive actions (e.g., decryption) require authentication via PGP key
* No password is stored or cached; all secrets are decrypted **in-memory only**

---

## Supported JSON-RPC Commands

### 🔧 Vault Management

* `diagnose`: Validates vault structure, permissions, metadata, and checksums
* `fix`: Attempts to correct permissions, regenerate metadata, and fix structure

### Secret Management

* `create`: Adds a new encrypted file and `.meta.toml`
* `edit`: Updates secret contents and/or metadata
* `read`: Returns decrypted secret and metadata
* `delete`: Removes both `.pgp` and `.meta.toml`
* `move`: Renames or relocates the secret
* `copy`: Duplicates a secret and its metadata
* `clone`: Re-encrypts the secret with a provided **public key**

### Utilities

* `find`: Lists secrets as a directory tree (filterable by tag, category, etc.)

---

## Security Model

* Single user per server instance
* Encryption via OpenPGP
* Secrets are decrypted **only in memory**
* TLS can be enabled for secure local HTTPS
* API access requires **PGP-based authentication**
* No password or master key storage (trust-based model)
* No soft deletes — deletions are **permanent**

---

## Diagnostics

* **Missing or invalid metadata**: flagged during `diagnose`
* **Broken or mismatched checksums**: flagged as **critical**
* All operations are executed via a **serialized queue** to prevent concurrency
  issues
