# listent Threat Model

Last updated: 2025-09-23  
Branch / Context: `003-add-launchd-daemon` (daemon + monitoring features)  
Scope: CLI scan mode, monitoring mode, LaunchD daemon, IPC, configuration, logging, release artifacts.

---
## 1. System Overview

`listent` is a macOS command-line tool and optional background daemon that:
- Scans directories (default: `/Applications`) for binaries and extracts code signing entitlements.
- Monitors running processes, reporting newly observed ones and their entitlements.
- Runs as a LaunchD-managed daemon for continuous monitoring.
- Exposes a Unix domain socket for runtime control (config updates, status, shutdown).
- Produces human-readable or JSON output; daemon logs via Unified Logging System (ULS).
- Manages configuration via a TOML file (e.g., `/etc/listent/daemon.toml`), and runtime artifacts: PID file, socket, LaunchD plist.

### 1.1 Architecture Components
| Component | Responsibility |
|-----------|----------------|
| CLI (`main`, `cli/`) | Parse args, dispatch modes |
| Scan (`scan/`) | Directory traversal, file filtering |
| Entitlements (`entitlements/`) | Extract & filter entitlement key/values |
| Monitoring (`monitor/`) | Poll process table, detect new processes |
| Daemon (`daemon/`) | LaunchD integration, IPC server, config lifecycle |
| IPC (`daemon/ipc.rs`) | Message framing, serde-based request/response |
| Config (`daemon/config.rs`) | Load/validate/merge TOML config |
| Logging (`daemon/logging.rs`, `monitor/unified_logging.rs`) | Structured event emission to ULS |
| Output (`output/`) | Human + JSON formats |
| Models (`models/`) | Shared data structures |

### 1.2 Data Flow (High-Level)
```
User/LaunchD -> CLI -> (Scan | Monitor Loop) -> Entitlement Extraction -> Output/Logging
Daemon IPC Client -> Unix Socket -> IPC Server -> (Config Update | Status | Shutdown) -> Logging
Config File -> Loader -> Validated In-Memory Config -> (Monitor Loop Parameters)
```

---
## 2. Assets

### 2.1 Primary Assets
- Integrity of scan / monitoring results (no falsified entitlements)
- Availability of daemon monitoring service
- Integrity of configuration (intervals, filters)
- Integrity & authenticity of released binary
- Accuracy of entitlement extraction

### 2.2 Confidentiality-Related
- Entitlement sets (can reveal capabilities: network, debug, TCC categories)
- Configuration contents (paths, filters)
- Logs (process names, paths)

### 2.3 Supporting Assets
- IPC control channel (prevent unauthorized changes)
- Runtime artifacts: PID file, socket file, LaunchD plist
- Dependency lock file (`Cargo.lock`)

---
## 3. Threat Actors
| Actor | Capability | Motivation |
|-------|------------|------------|
| Local unprivileged user | Access within user space | Disable / tamper / gain intel |
| Malicious process owner | Control over own binary/process | Evade detection; mislead monitoring |
| Partial FS attacker | Write to monitored dirs | Trigger DoS / confuse scanning |
| Supply chain adversary | Package/build compromise | Distribute trojanized binary |
| Privileged attacker (root) | Full system control | Typically out-of-scope (residual) |
| Malware (automated) | Opportunistic local logic | Disable monitoring / resource drain |

---
## 4. Trust Boundaries
1. Filesystem enumeration (untrusted directory contents)
2. Process enumeration (OS APIs -> internal state)
3. Entitlement extraction (untrusted codesign metadata)
4. IPC (Unix socket clients)
5. Configuration load/update (disk -> runtime)
6. LaunchD interaction (plist install/remove)
7. Output/log formatting (internal -> external consumers)
8. Runtime artifacts creation (PID, socket path)

---
## 5. Detailed Data Flows

### 5.1 Scan Mode
Args -> Validate -> Collect candidate files -> Filter -> Extract entitlements -> Aggregate -> Output -> Exit

### 5.2 Monitor Loop
Config (interval, filters) -> Enumerate processes -> Diff vs cached snapshot -> For new PIDs extract entitlements -> Emit events/logs -> Sleep -> Repeat

### 5.3 Daemon IPC
Client connects -> Handshake (implicit) -> Deserialize request -> Authorization (filesystem perms) -> Execute (update config/status) -> Serialize response -> Log result

### 5.4 Config Update
IPC Update -> Validate candidate config -> Write temp -> fsync -> Atomic rename -> Replace in-memory -> Backup old -> Audit log entry

---
## 6. Attack Surface Inventory
| Surface | Examples |
|---------|----------|
| CLI arguments | Extremely low intervals; massive filter lists |
| Filesystem traversal | Deep recursion, symlink loops, huge file counts |
| Entitlement extraction | Malformed Mach-O / signature data causing parser stress |
| IPC socket | Unauthorized config mutation, flooding, large messages |
| Config file | Tampering, invalid values, oversize arrays |
| LaunchD plist | Replacement, privilege escalation attempts |
| Output/Logs | Control character injection, misleading entries |
| Runtime artifacts | Symlink substitution for PID/socket |
| Dependencies | Malicious crate updates |

---
## 7. STRIDE Analysis (Expanded)

### 7.1 Filesystem Enumeration
- Spoofing: Symlink disguises path identity
- Tampering: Replacing scanned binaries during scan window
- Repudiation: Lack of per-file audit trail
- Information Disclosure: Reading entitlements of binaries revealing capabilities
- DoS: Deep directory trees, millions of nodes
- Elevation: (Not directly applicable)

### 7.2 Process Monitoring
- Spoofing: PID reuse with different executable
- Tampering: Rapid spawn/kill to obscure presence
- Repudiation: Missing event sequence numbers
- Info Disclosure: Listing processes with sensitive names/paths
- DoS: Process churn overwhelming loop

### 7.3 Entitlement Extraction
- Tampering: Crafted Mach-O altering parsing flows
- DoS: Oversized entitlement sections
- Info Disclosure: Sensitive private entitlements displayed

### 7.4 IPC Socket
- Spoofing: Any user connects if permissive perms
- Tampering: Config changes without authorization
- Repudiation: No audit of who changed what (no identity metadata)
- DoS: Flood connections; large payload messages
- Elevation: Setting interval to extremely low values for resource abuse

### 7.5 Config Management
- Tampering: Direct file edit bypassing validation
- Repudiation: Unlogged changes
- DoS: Oversized arrays causing memory spike

### 7.6 LaunchD Integration
- Tampering: Replaced plist to point to modified binary
- DoS: Misconfigured keep-alive loops
- Elevation: Running under more privileged user than needed

### 7.7 Logging/Output
- Tampering: Insertion of control characters for log poisoning
- Repudiation: Lack of event signatures
- Info Disclosure: Overly verbose logs with sensitive paths

### 7.8 Supply Chain
- Tampering: Malicious dependency upgrade
- Spoofing: Fake release asset distribution

---
## 8. Threats & Mitigations Matrix
| Threat | Risk Level | Mitigation (Current / Planned) |
|--------|------------|--------------------------------|
| Symlink attack on socket/PID | High | Use `lstat`, reject symlink, `O_CREAT|O_EXCL`, perms 0600 (Planned) |
| Unauthorized IPC access | High | Restrictive perms now; future: group membership or shared secret auth |
| Config tampering | High | Validation at load; planned atomic updates + hash log |
| Resource exhaustion (interval) | High | Interval bounds enforced (0.1–300); clamp & log |
| Massive directory traversal | High | Planned depth & file count guard + early abort warning |
| Malformed binary crash | Medium | Graceful error path; planned fuzz & caps |
| PID reuse spoofing | Medium | Track path; planned inode/ctime correlation |
| IPC flooding | Medium | Planned rate limit, max connections, message size cap |
| Log injection | Medium | Planned control char sanitization |
| Supply chain compromise | High | Commit lockfile; planned signing, SBOM, provenance |
| Entitlement privacy leak | Medium | Potential redaction mode (future) |
| Config rollback failure | Medium | Backups + atomic rename (Planned) |

---
## 9. Validation & Defensive Design

### 9.1 Input Validation Principles
- Fail fast: invalid arg/interval aborts before operations
- Deny unknown config fields via `serde(deny_unknown_fields)`
- Canonicalize paths then check scope

### 9.2 Resource Controls
- Interval clamp >=0.1s
- Entitlement list length & key length caps (planned constants)
- Pre-allocate & reuse collections in monitor loop

### 9.3 Error Handling
- Favor `Result` propagation with context
- Convert entitlement extraction failures to structured error event (not crash)

### 9.4 Logging Hygiene
- Distinct categories: `audit`, `monitor`, `error`
- Escape control characters (e.g., below 0x20 except tab/newline) before emission
- Include sequence numbers for ordering (planned)

---
## 10. Configuration Hardening Workflow (Planned)
1. Parse new TOML -> structured model
2. Validate numeric bounds, path canonicalization, list sizes
3. Write to temporary file `<cfg>.new` with perms 0600
4. `fsync` file & directory
5. Atomic rename over original
6. Create backup `<cfg>.bak.<timestamp>` including SHA256 hash
7. Emit audit log: old_hash -> new_hash
8. Replace in-memory config reference

Edge Cases: Partial write failure, disk full, permission error -> revert & log error.

---
## 11. IPC Security Strategy
- Restrictive file permissions (0600) on socket path
- Socket directory ownership root or deploying user
- Future: simple auth token file (0600) read by client + server handshake
- Per-connection read timeout (e.g., 2s)
- Maximum message size (e.g., 64 KB)
- Bounded concurrent connections (e.g., 16)
- Rate limiting (token bucket per source UID if obtainable)

---
## 12. Entitlement Extraction Hardening
| Concern | Control |
|---------|---------|
| Oversized entitlement blob | Cap processed size, truncate with indicator |
| Malformed Mach-O structure | Parser returns error; isolate from crash |
| CPU intensive parsing | Early abort on size thresholds |
| Memory bloat | Streaming parse / pre-sized buffers |

Fuzz Plan: Use `cargo-fuzz` with corpus of real Mach-O headers + mutated entitlement plists.

---
## 13. Supply Chain Security
| Control | Status |
|---------|--------|
| `Cargo.lock` committed | Yes |
| `cargo audit` in CI | Planned |
| Dependency allow/deny list (`cargo deny`) | Planned |
| Release codesigning + notarization | Partial (verify pipeline) |
| SHA256 checksum publication | Planned |
| SBOM (CycloneDX) generation | Planned |
| Provenance attestation (SLSA) | Future |

---
## 14. Testing Strategy (Security Augmentation)
| Test Type | Examples |
|-----------|----------|
| Unit | Interval clamp, path canonicalization, entitlement truncation |
| Integration | IPC flood simulation, config update rollback, symlink runtime artifact creation |
| Property | Path normalization invariants, entitlement key parser |
| Fuzz | Entitlement parser, IPC message deserializer |
| Regression | Specific CVE-like scenarios after fixes |

---
## 15. Metrics & Observability (Future)
| Metric | Purpose |
|--------|---------|
| Process enumeration duration | Detect performance regressions |
| Extraction failures per interval | Spot systemic parsing issues |
| IPC request rate | Detect abuse/flood |
| Config change count | Audit anomaly detection |

Expose via IPC `GetStats` response; optionally integrate with local tooling.

---
## 16. Privacy Considerations
- Entitlements may reveal user/system capability usage patterns
- No network transmission of data; local only
- Planned `--redact-sensitive` to hash specific entitlement key patterns
- Documentation must clarify data retention scope (currently in-memory only unless user saves JSON)

---
## 17. Residual Risks
| Risk | Rationale |
|------|-----------|
| Local privileged (root) bypass | Out-of-scope; root can modify binary / runtime |
| PID reuse race | Time-of-check vs time-of-use inherent in polling |
| Socket unauthorized if perms mis-set | Depends on deployment correctness |
| Supply chain ecosystem trust | Relies on crates.io integrity & transitive deps |
| Parser denial via novel Mach-O quirks | Mitigated by fuzzing but never zero |

---
## 18. Roadmap (Security-Focused)
| Phase | Item |
|-------|------|
| Immediate | Symlink defense, interval enforcement test coverage |
| Short Term | Atomic config + backups + audit logs |
| Short Term | IPC rate limiting + message size cap |
| Mid Term | Fuzz harness & crash triage CI gating |
| Mid Term | Code signature verification of monitored binaries |
| Mid Term | Entitlement truncation + redaction mode |
| Long Term | Output signing, provenance attestations |
| Long Term | Sandboxed extraction helper process |

---
## 19. Executive Summary
`listent` analyzes local binaries and active processes—its trust boundary is entirely local, making integrity and availability the foremost concerns. Key near-term improvements: (1) secure runtime artifact creation, (2) authenticated/rate-limited IPC, (3) atomic & audited configuration changes, (4) release supply chain integrity (signing + SBOM), and (5) parser robustness via fuzzing. Implementing these yields substantial reduction in tampering and DoS risk while preserving performance.

---
## 20. Maintenance & Update Triggers
Update this document when:
- IPC protocol changes
- Config schema adjusts
- New privileged operations or security-relevant features added
- Release signing / provenance pipeline changes
- Entitlement extraction logic materially updated

---
## 21. Change Log
| Date | Change |
|------|--------|
| 2025-09-23 | Initial expanded threat model added |

---
## 22. Glossary
| Term | Definition |
|------|------------|
| PID | Process Identifier |
| IPC | Inter-Process Communication (Unix Domain Socket here) |
| ULS | Unified Logging System (macOS) |
| SBOM | Software Bill of Materials |
| SLSA | Supply-chain Levels for Software Artifacts framework |

---
## 23. Acknowledgments
Initial model drafted via internal review and automated assistant; maintainers responsible for ongoing accuracy.
