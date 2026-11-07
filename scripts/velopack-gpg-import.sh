#!/usr/bin/env bash
# Import the ShellZip release OpenPGP key for detach-signing Linux artifacts.
# Expects GPG_PRIVATE_KEY (armored) and optional GPG_PASSPHRASE in the environment.
set -euo pipefail

if [[ -z "${GPG_PRIVATE_KEY:-}" ]]; then
  echo "velopack-gpg-import: GPG_PRIVATE_KEY is empty; skipping import" >&2
  exit 0
fi

GNUPGHOME="${GNUPGHOME:-$(mktemp -d)}"
export GNUPGHOME
chmod 700 "$GNUPGHOME"
echo "$GPG_PRIVATE_KEY" | gpg --batch --import
if [[ -n "${GPG_PASSPHRASE:-}" ]]; then
  echo "velopack-gpg-import: passphrase configured for loopback signing"
fi
echo "GNUPGHOME=$GNUPGHOME"
