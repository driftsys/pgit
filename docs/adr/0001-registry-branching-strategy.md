# ADR-0001: Registry Branching Strategy

## Status
Proposed

## Context

A pgit registry is a git repository. The branching model determines how packages and
versions are stored inside it, which cascades into: fetch size, history isolation, LFS
scoping, tag resolution, index maintenance, and publish complexity.

---

## Options

### A — Monorepo trunk

All packages live as subdirectories on `main`.

```
main
├── registry.toml
├── packages/
│   ├── tool-a/    ← all versions collapsed into one directory
│   └── tool-b/
```

Tags point to commits on `main`:
```
tool-a@1.0.0  →  commit on main
tool-a@1.1.0  →  commit on main
```

**Pros:**
- Simplest model — standard git, no special branch management
- One clone gets the full catalog
- Trivial to browse on GitHub/GitLab UI

**Cons:**
- Fetching one package downloads every package's history and files
- All LFS objects are in one namespace — no scoping per package
- History of tool-a and tool-b is entangled
- Large registries become expensive to clone even shallowly

---

### B — Star branches (feature-style)

Each package version lives on a short-lived branch, merged to `main` via PR.

```
main              ← merged history of all packages
pkg/tool-a/1.0.0  ← branch, merged and deleted after release
pkg/tool-a/1.1.0  ← branch, merged and deleted after release
```

**Pros:**
- PRs provide review/approval gate before a version lands
- Familiar workflow for teams already using PR-based development

**Cons:**
- Same storage problem as monorepo after merge
- Branch proliferation — thousands of stale branches for active registries
- Approval workflow belongs to governance layer, not storage model
- Adds ceremony without storage benefit

---

### C — Orphan lines

Each package lives on a permanent orphan branch with no shared history with `main`.
Tags point to commits on the orphan branch.

```
main              ← registry.toml + index.toml only
pkg/tool-a        ← orphan: tool-a files only, all versions as commits
pkg/tool-b        ← orphan: tool-b files only
```

Tags:
```
tool-a@1.0.0  →  commit on pkg/tool-a
tool-a@1.1.0  →  commit on pkg/tool-a
tool-b@1.0.0  →  commit on pkg/tool-b
```

Fetch:
```bash
git clone --depth 1 --branch tool-a@1.0.0 <url>
# downloads only tool-a files — nothing from tool-b
```

**Pros:**
- Minimal fetch: one package = one shallow clone of one branch
- Complete history isolation per package
- LFS objects scoped to each branch — only pulled when that package is fetched
- `archive` mode maps naturally: orphan branch is append-only by design

**Cons:**
- pgit must manage orphan branch creation and tag placement
- Index on `main` must be kept in sync with orphan branch state
- GitHub/GitLab UI shows many branches — less browsable
- `git clone` of the whole registry requires fetching all orphan branches explicitly

---

### D — Hybrid: orphan lines + packed index

Orphan lines for content (as in C), but the index on `main` is a generated artifact
(e.g., `index.toml` or `index.json`) rebuilt on every publish, functioning as a
read-only catalog cache.

Consumers who only need to browse the catalog clone `main` (tiny).
Consumers who need a package clone the specific orphan branch tag (minimal).

This is identical to C but makes the index role explicit.

---

## Comparison

| Criterion               | A — Monorepo | B — Star   | C — Orphan | D — Hybrid |
|-------------------------|-------------|------------|------------|------------|
| Fetch size per package  | whole repo  | whole repo | minimal    | minimal    |
| History isolation       | no          | no         | yes        | yes        |
| LFS scoping             | none        | none       | per branch | per branch |
| Index maintenance       | implicit    | implicit   | explicit   | explicit   |
| Archive mode fit        | weak        | weak       | strong     | strong     |
| UI browsability         | good        | poor       | fair       | fair       |
| Publish complexity      | low         | medium     | high       | high       |

---

## Open Questions

1. Should pgit support both A and C (registry declares its layout), or mandate one?
2. In orphan model: does the tag live on the orphan branch commit, or is it a global tag
   pointing at that commit? (Git tags are global — does this create conflicts across packages
   sharing a version string like `@1.0.0`?)
3. For single-package repos (no `registry.toml`): are they layout A with one package,
   or a degenerate case outside the layout model entirely?
4. In layout D: who is authoritative when `index.toml` on `main` diverges from the actual
   orphan branch tags? (e.g. after a force operation or partial publish failure)
