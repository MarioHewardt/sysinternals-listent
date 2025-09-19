# Human Output Format Contract

## Result Entry Format
```
<path>
  entitlement: <key>=<value>
  entitlement: <key>=<value>
```

- Consecutive results separated by a single blank line.
- Values displayed as JSON-style scalar (true/false/number) or quoted string if contains whitespace.
- Ordering: entire result set sorted lexicographically by path.

## Skipped / Warnings (Non-Quiet Mode)
```
WARN unreadable: <path> (<reason>)
```
Printed to stderr only.

## Summary Block (if enabled)
```
---
Scanned: <n>
Matched: <m>
Skipped (unreadable): <s>
Duration: <t_ms>ms
Interrupted: yes (only if true)
```

## Quiet Mode
- Suppresses WARN lines.
- Summary still printed unless --no-summary specified.

## No Matches Case
```
(no matches)
```
Followed by summary (unless suppressed).

## Interrupt Behavior
- If interrupted after some results: ensure summary prints with Interrupted: yes.
- If no results yet and interrupted: still print summary with Matched: 0.
