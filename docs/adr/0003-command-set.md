# ADR-0003: Command Set

## Status
Proposed

## Context

pgit needs a command set that covers the full package management lifecycle:
registry setup, publishing, consuming, verifying, and maintaining. The commands
must work consistently across both distribution and archive modes and map cleanly
onto the underlying git operations.

---

## Core commands (undisputed)

These are required regardless of other decisions.

| Command | Description |
|---------|-------------|
| `pgit init` | Initialise a new registry repository |
| `pgit add` | Install a package from a registry |
| `pgit publish` | Publish a package version to a registry |
| `pgit list` | List packages (in a registry or installed locally) |
| `pgit verify` | Verify installed packages against recorded hashes |

---

## Disputed / optional commands

### `pgit remove` vs `pgit yank`

Two distinct operations that are often conflated:

- **remove**: deletes the package content from the registry entirely (destructive)
- **yank**: marks a version as deprecated/unsafe without removing it — consumers
  who already have it keep working, new installs are warned or blocked

| | `remove` | `yank` |
|--|---------|--------|
| Content deleted | yes | no |
| Existing installs break | yes | no |
| New installs blocked | yes | warn or block |
| Allowed in archive mode | no | warn-only |

**Option A**: expose both as separate commands.
**Option B**: expose only `yank`; `remove` is an admin-only operation via git directly.

---

### `pgit update`

Bumps installed packages to newer versions, respecting semver constraints in the lock.

**Questions:**
- Does it update to latest, or to latest-within-constraint?
- In archive mode, `update` resolves the new SHA and writes it — but does it warn
  that the previous entry is now superseded?
- Does `--dry-run` show a diff of what would change?

---

### `pgit search`

Full-text or name search across a registry's index.

**Requires:** index on `main` to be searchable without cloning all orphan branches.

**Option A**: `pgit search <query> <registry>` — queries the local index cache.
**Option B**: `pgit search <query>` — searches across all configured registries.
**Option C**: omit for now; discoverable via `pgit list`.

---

### `pgit archive`

Create an immutable local snapshot of a specific package version for offline/regulatory storage.

```bash
pgit archive pkg:pgit/github.com/acme/registry/tool-a@1.0.0 --out ./snapshots/
# writes: tool-a-1.0.0.tar.gz + tool-a-1.0.0.provenance.json
```

**Question:** Is this a separate command, or a flag on `pgit add` (`--snapshot`)?

---

### `pgit lock` / `pgit unlock`

Explicitly freeze or unfreeze a local installation at its current resolved SHA.

**Overlap with verify:** `pgit verify` already detects drift. Is an explicit lock command
needed, or does the lock file managed by `add`/`update` cover this?

---

### `pgit info` / `pgit show`

Display metadata about an installed or remote package:
- Source PURL
- Resolved SHA
- Content hash
- Published at
- Mode (distribution / archive)

**Question:** Is this covered by `pgit list --verbose`, or does it warrant its own command?

---

## Proposed minimal command set (v1)

```
pgit init     --name <name> [--archive] [--lfs]
pgit add      <purl-or-shorthand> [--out <dir>] [--ref <ref>] [--token <tok>]
pgit publish  <path> --to <registry> --version <ver> [--release] [--token <tok>]
pgit list     [<registry>] [--installed]
pgit verify   [<names>...] [--strict]
pgit yank     <purl> [--reason <str>] [--token <tok>]
pgit update   [<names>...] [--dry-run]
pgit info     <purl-or-name>
pgit archive  <purl> --out <dir>
```

`search` deferred to v2 (depends on index format decision from ADR-0001).

---

## Open Questions

1. Should `pgit add` accept multiple sources at once (`pgit add a b c`)?
2. Should there be a `pgit registry` subcommand namespace for registry-level operations
   (`pgit registry init`, `pgit registry protect`, `pgit registry list`)?
3. Does `pgit publish` in archive mode require an additional `--confirm` flag to make
   the irreversibility explicit?
4. Should `pgit update` be a no-op in archive mode (consumers pin SHAs, updates are
   meaningless) or should it resolve newer versions in the same registry?
