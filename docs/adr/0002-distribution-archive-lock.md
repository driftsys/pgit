# ADR-0002: Distribution vs Archive Modes and Lock Behavior

## Status
Proposed

## Context

pgit registries serve two distinct purposes that have different immutability guarantees:
- **Distribution**: active tooling, scripts, skills — can be updated, yanked, superseded
- **Archive**: regulatory, compliance, audit — immutable once written, append-only

The lock behavior (what gets recorded when a consumer installs a package) must align with
the registry mode to provide meaningful guarantees.

Additionally, GitHub and GitLab both offer native platform features that can enforce
immutability at the infrastructure level, independent of pgit client-side checks.

---

## Part 1 — Registry and Package Modes

### Option A — Registry-level mode only

`registry.toml` declares one mode for all packages. No per-package override.

```toml
[registry]
mode = "archive"
```

**Pros:** Simple, unambiguous, no inheritance complexity.
**Cons:** A registry cannot mix archived releases with active distribution packages.

---

### Option B — Registry floor + per-package override

Registry declares a default. Individual packages can declare a stricter mode.
Downgrading below the registry floor is refused.

```toml
# registry.toml
[registry]
mode = "distribution"   # floor

# packages/certified-tool/package.toml
[package]
mode = "archive"        # stricter than floor — allowed

# packages/other-tool/package.toml
[package]
mode = "distribution"   # same as floor — allowed

# In an archive registry, this would be refused:
# mode = "distribution"  ← error: cannot downgrade below registry floor
```

Resolution table:

| Registry | Package | Effective |
|----------|---------|-----------|
| distribution | unset | distribution |
| distribution | distribution | distribution |
| distribution | archive | archive |
| archive | unset | archive |
| archive | archive | archive |
| archive | distribution | **refused** |

**Pros:** Flexible; a single registry can host both tooling and certified artifacts.
**Cons:** Requires mode resolution logic in pgit and in platform lock enforcement.

---

## Part 2 — Lock Behavior

When a consumer runs `pgit add`, what gets recorded in the local lock?

### Option 1 — Tag only

```json
{ "name": "tool-a", "source": "github.com/acme/registry", "ref": "1.0.0" }
```

**Pros:** Human-readable, easy to audit visually.
**Cons:** Tags are mutable (can be force-moved). Provides no integrity guarantee.

---

### Option 2 — Resolved commit SHA

```json
{ "name": "tool-a", "ref": "1.0.0", "resolved_sha": "abc123def456..." }
```

Tag is resolved to its commit SHA at install time and pinned.

**Pros:** Immutable reference — SHA cannot be faked or moved.
**Cons:** SHA alone doesn't prove content integrity (tree could differ if history was rewritten).

---

### Option 3 — Content hash

```json
{ "name": "tool-a", "ref": "1.0.0", "content_hash": "sha256:deadbeef..." }
```

SHA-256 of the package file tree, computed after download.

**Pros:** Detects tampering or corruption regardless of git history.
**Cons:** Does not prove the content came from the expected commit.

---

### Option 4 — Full provenance (recommended for archive mode)

```json
{
  "name": "tool-a",
  "source": "pkg:pgit/github.com/acme/registry/tool-a@1.0.0",
  "ref": "1.0.0",
  "resolved_sha": "abc123def456...",
  "content_hash": "sha256:deadbeef...",
  "installed_at": "2026-04-02T10:00:00Z"
}
```

**Pros:** Both SHA and content hash — covers git history rewrite AND file tampering.
**Cons:** More data; `pgit verify` must recompute content hash on demand.

---

### Lock strictness by mode

| Mode | Minimum lock requirement |
|------|--------------------------|
| distribution | tag + content_hash |
| archive | tag + resolved_sha + content_hash + installed_at |

---

## Part 3 — Platform Lock Enforcement (GitHub / GitLab)

Client-side pgit checks are advisory. For hard guarantees, GitHub and GitLab provide
infrastructure-level controls that can be configured per registry.

### GitHub

| Feature | Enforces |
|---------|----------|
| Protected tags (`tool-a@*`) | prevents tag deletion and force-move |
| Branch protection on `pkg/tool-a` | prevents force-push to orphan branch |
| Rulesets (tag protection) | same as above, more granular |
| `CODEOWNERS` | requires review before publish lands |
| Audit log (Enterprise) | records every push, tag create/delete |

Configuration pgit can emit on `pgit init --archive`:
```bash
gh api repos/{owner}/{repo}/branches/main/protection \
  --method PUT --field required_status_checks=null \
  --field enforce_admins=true \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

Tag protection via ruleset:
```bash
gh api repos/{owner}/{repo}/rulesets --method POST \
  --field name="protect-release-tags" \
  --field target="tag" \
  --field enforcement="active" \
  --field conditions='{"ref_name":{"include":["refs/tags/*@*"],"exclude":[]}}' \
  --field rules='[{"type":"deletion"},{"type":"non_fast_forward"}]'
```

### GitLab

| Feature | Enforces |
|---------|----------|
| Protected tags | prevents tag deletion and force-move |
| Protected branches | prevents force-push to orphan branches |
| Push rules | enforce commit format, signed commits |
| Audit events (Premium+) | records all push events |

---

## Open Questions

1. Should `pgit init --archive` automatically configure platform protections via the
   GitHub/GitLab API, or document them and let the operator apply manually?
2. Should the lock file be a separate `pgit.lock` or embedded in an existing project
   manifest (e.g., alongside `package.toml`)?
3. For `distribution` mode: should yanked versions remain resolvable with a warning,
   or be hard-removed from the index?
4. Should pgit support signed commits/tags (GPG or SSH signing) as an additional
   integrity layer, and if so, is verification mandatory or optional?
