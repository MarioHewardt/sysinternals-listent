# Security Policy

Project: `listent`  
Last Updated: 2025-09-23  
Applies To: CLI scan mode, monitor mode, LaunchD daemon, IPC, configuration, logging, release artifacts.

---

## 1. Supported Versions

| Version Track | Status | Security Fixes |
|---------------|--------|----------------|
| `main` / active feature branches | Development | Best-effort |
| Latest tagged release (N) | Supported | Yes |
| N-1 release | Supported (grace) | Yes (critical only) |
| Older (< N-1) | EOL | No |

(Adjust after formal releases.)

---

## 2. Reporting a Vulnerability

If you discover a security issue:

1. Do **not** open a public GitHub issue.
2. Email: `SECURITY_CONTACT_PLACEHOLDER` (replace with maintainer contact or use GitHub Security Advisories).
3. Provide:
   - Commit hash or release version
   - Reproduction steps or PoC
   - Impact assessment (confidentiality / integrity / availability)
   - Any logs or sanitized output
4. You should receive acknowledgment within **5 business days**.
5. Coordinated disclosure: we prefer a 14–30 day remediation window depending on severity.

Future improvements:
- Add a security advisory template
- Provide a PGP key fingerprint for encrypted reports

---

## 3. Preferred Communication

| Purpose | Channel |
|---------|---------|
| Vulnerability report | Private email / GH advisory |
| Clarification (non-sensitive) | GitHub issue |
| Responsible disclosure coordination | Email thread |

---

## 4. High-Level Security Posture

`listent` processes untrusted local data from:
- The filesystem (binary discovery)
- Running processes (enumeration)
- Code signing metadata (entitlement extraction)
- Local IPC clients (Unix domain socket control commands)

Primary protection goals:  
1. Integrity of entitlement scan & monitoring results  
2. Availability of the daemon monitoring loop  
3. Integrity & controlled mutation of configuration  
4. Authenticity & integrity of distributed binaries (supply chain)  
5. Controlled access to IPC operations (prevent unauthorized shutdown or config changes)

---

## 5. Key Risks (Top 10)

| # | Risk | Impact | Current Mitigation | Planned |
|---|------|--------|--------------------|---------|
| 1 | Unauthorized IPC config changes | Disable / degrade monitoring | Filesystem perms | Socket auth/rate limit |
| 2 | Symlink attacks on PID/socket | File overwrite or hijack | Basic path control | Symlink-safe create (O_EXCL) |
| 3 | Resource exhaustion (tiny interval / huge tree) | CPU / memory spike | Interval bounds (0.1–300) | File count / depth caps |
| 4 | Malformed binary causing parser crash | Monitoring gap | Error handling | Fuzz + isolation |
| 5 | Supply chain compromise | Trojaned binary | Lockfile, review | Signing + SBOM + provenance |
| 6 | Log / output injection (control chars) | Downstream confusion | Limited sanitization | Full escaping |
| 7 | PID reuse spoofing | False attribution | Basic tracking | Inode/ctime correlation |
| 8 | Config tampering | Silent behavior change | Bounds checking | Atomic updates + hash log |
| 9 | IPC flooding | Denial of service | None (baseline) | Rate & connection limits |
| 10 | Entitlement privacy exposure | Unintended disclosure | Local only | Redaction mode |

---

## 6. Threat Model (Condensed)

### Trust Boundaries
- Filesystem → scanner
- OS process table → monitor
- Code signing metadata → entitlement parser
- Unix domain socket → daemon control interface
- Config file → runtime configuration

### STRIDE Summary (Per Boundary)
| Boundary | Key STRIDE Concerns |
|----------|---------------------|
| Filesystem | Tampering, DoS (path explosion), spoofing via symlinks |
| Process Enumeration | Spoofing (PID reuse), DoS (churn) |
| Entitlement Extraction | Tampering (crafted binary), DoS (parser stress) |
| IPC | Spoofing (unauthorized client), DoS (flood), Tampering (config) |
| Configuration | Tampering, Repudiation (no audit), DoS (oversized values) |

### Core Mitigations (Target State)
- Secure file creation (reject symlinks, restrictive perms)
- Interval & list size clamping + canonical path filters
- IPC rate limiting + optional authentication
- Atomic config updates with backup & hash-based audit
- Entitlement parser fuzzing & length caps
- Release integrity (codesign + notarization + checksums + SBOM)
- Structured logging with control character sanitization

---

## 7. Configuration Security Guidelines

| Aspect | Guideline |
|--------|-----------|
| File perms | Use `0600` (or `0640` root:trusted group) |
| Updates | Validate → write temp → fsync → atomic rename |
| Backups | Keep N previous + SHA256 hash |
| Validation | Reject relative paths & duplicates; clamp intervals; cap list sizes |

---

## 8. IPC Security Guidelines

| Concern | Control |
|---------|---------|
| Unauthorized access | Socket directory perms + mode `0600` |
| Flood / DoS | (Planned) rate limiting & bounded connection count |
| Message abuse | Max message size; strict serde schema |
| Auditing | Log config changes & denied attempts (no payload echo) |

---

## 9. Logging & Output Hygiene

- Strip or escape control characters before emission.
- Separate channels: `audit`, `monitor`, `error`.
- Avoid logging raw entitlement lists when redaction mode (future) is enabled.
- Consider adding monotonic sequence numbers for tamper-evidence.

---

## 10. Supply Chain & Build Integrity

Planned / Recommended:
- Commit `Cargo.lock`
- CI: `cargo audit` & `cargo deny`
- `rust-toolchain.toml` pinning
- Codesign + Apple notarization for macOS binaries
- Publish SHA256 checksums + CycloneDX SBOM
- (Future) Provenance attestation (SLSA, Sigstore/Cosign)

---

## 11. Testing Strategy (Security-Focused)

| Test Type | Coverage |
|-----------|----------|
| Unit | Interval bounds, path canonicalization, entitlement caps |
| Integration | IPC misuse, config rollback, large directory scenarios |
| Property | Path normalization, entitlement parsing edge cases |
| Fuzz (planned) | Entitlement parser, IPC message deserialization |
| Regression | Symlink handling, malformed binary resilience |

---

## 12. Roadmap Snapshot

| Phase | Focus |
|-------|-------|
| Immediate | Symlink-safe runtime artifacts, stricter validation |
| Short Term | Atomic config updates, IPC hardening, audit logs |
| Mid Term | Fuzzing, code signature validation, rate limiting |
| Long Term | Output signing, provenance, parser sandboxing, policy engine |

---

## 13. Residual Risks

- Local privileged attacker (root) can bypass safeguards.
- Race conditions (PID reuse) reduced, not eliminated.
- Without authentication beyond filesystem permissions, socket misuse still possible if perms misconfigured.
- Supply chain risk persists without full reproducible build attestation.

---

## 14. Coordinated Disclosure Timeline (Example)

| Day | Action |
|-----|--------|
| 0 | Report received & acknowledged |
| 1–5 | Triage & severity assignment |
| ≤14 | Patch development + review |
| ≤21 | Release prepared + advisories drafted |
| ≤30 | Public disclosure (unless extended by reporter agreement) |

---

## 15. Contact & Metadata

| Item | Value |
|------|-------|
| Security Contact | SECURITY_CONTACT_PLACEHOLDER |
| GPG Key | (Planned – publish fingerprint) |
| Issue Tracker | Public (non-sensitive only) |
| Advisory Channel | GitHub Security Advisories (planned) |

---

## 16. Quick Checklist

| Control | Status |
|---------|--------|
| Interval bounds validation | Implemented |
| Path filter canonicalization | Verify / finalize |
| Symlink-safe socket & PID creation | Planned |
| Atomic config updates + backups | Planned |
| IPC auth / rate limiting | Planned |
| Logging control char sanitization | Planned |
| Entitlement size caps | Planned |
| Release signing & notarization | Partial / verify |
| SBOM & checksums | Planned |
| Fuzzing (entitlement parser) | Planned |
| Code signature verification | Future |
| Output signing | Future |

(Keep table updated with each release.)

---

## 17. Change Log (Security Doc)

| Date | Change |
|------|--------|
| 2025-09-23 | Initial `SECURITY.md` creation with embedded threat model summary |

---

## 18. Attribution

Initial threat model and policy drafted collaboratively via internal review and automated assistant support. Maintainers should revise as architecture evolves.

---
