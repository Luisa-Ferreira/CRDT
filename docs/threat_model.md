# Threat Model v0 — NetGuardian CRDT Service

## Context
Distributed file synchronization service based on CRDT, written in Rust.

## Assets
- User data/files
- CRDT state (ops, frontier)
- Node private keys
- Signed snapshots

## Adversaries
- External attacker (network attack/MITM)
- Insiders with compromised node
- Ransomware encrypting local storage

## Attack surfaces
- Network endpoints (gRPC/HTTP)
- Local file system (state files)
- PKI / key provisioning script

## High-risk scenarios
1. Compromised node signs malicious operations
2. MITM injects invalid operations
3. Ransomware deletes/encrypts snapshots and state

## Controls
- mTLS with CA; client certificate validation
- Operations signed (Ed25519) and verified before applying
- Signed snapshots + restore verification
- Audit logs + rate limiting + quarantine