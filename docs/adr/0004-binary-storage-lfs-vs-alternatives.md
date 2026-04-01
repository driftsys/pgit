# ADR-0004: Binary Storage — LFS vs Alternatives

## Status
Proposed

## Context

pgit packages may contain binary files: compiled tools, tarballs, datasets, signed
artifacts. Git's object store handles text well but becomes expensive for large
binaries. Several strategies exist, each with different safety, portability, and
cost profiles.

The choice is particularly consequential for **archive mode**, where the storage
layer must provide long-term immutability guarantees that survive platform changes.

---

## The core safety question about LFS

Git LFS stores binary content **outside the git object store**, on a separate LFS
server (GitHub LFS, GitLab LFS, or self-hosted). The git repo holds only a pointer:

```
version https://git-lfs.github.com/spec/v1
oid sha256:deadbeef...
size 1048576
```

This breaks the fundamental git guarantee that content is self-contained and
content-addressed within the repository.

**Specific risks:**

| Risk | Severity |
|------|----------|
| LFS objects can be deleted from the server independently of git history | High for archive |
| GitHub/GitLab bandwidth quotas (1 GB/month free) — fetch fails silently above limit | Medium |
| LFS objects do not transfer automatically during repo migration | High for portability |
| Without `git-lfs` installed, clone silently returns pointer files, not content | Medium |
| Self-hosted LFS server is additional infrastructure to maintain | Low–Medium |

**Conclusion on safety**: LFS is **not safe for regulatory archives**. It is acceptable
for distribution registries where content can be re-fetched or reconstructed.

---

## Options

### A — Git LFS (opt-in)

Store large binaries in LFS, tracked via `.lfsconfig` patterns per package or registry.

```bash
# In orphan branch for pkg/tool-a:
git lfs track "*.tar.gz" "*.bin"
```

**Pros:**
- Native git workflow; transparent on clone if git-lfs is installed
- Works with existing GitHub/GitLab UI and CI

**Cons:**
- All risks above apply
- Not suitable for archive mode
- Adds `git-lfs` as a hard runtime dependency

---

### B — Release assets (GitHub Releases / GitLab Releases)

Binaries are attached to a git tag as release assets, stored and served by the
platform's release infrastructure (not in the git object store or LFS).

```
tag: tool-a@1.0.0
  └─ release asset: tool-a-1.0.0.tar.gz  (SHA256 in release notes or provenance.json)
```

`pgit add` downloads the asset via the platform API or a stable URL:
```
https://github.com/acme/registry/releases/download/tool-a@1.0.0/tool-a-1.0.0.tar.gz
```

**Pros:**
- No git-lfs dependency
- Stable download URL independent of git clone
- Release assets are explicitly part of the platform's release lifecycle
- Can be independently archived/mirrored (wget, curl, any HTTP client)
- Content hash is verifiable against `provenance.json` in the git tree

**Cons:**
- Binaries are outside the git object store — same decoupling risk as LFS
- GitHub/GitLab specific — self-hosted git servers need alternative
- Release asset deletion is possible by repo admins

---

### C — Binaries committed directly into git (small files only)

For small binaries (< 1–5 MB), commit them directly as git objects.
Git compresses and deduplicates across versions.

**Pros:**
- Content is fully self-contained in the git object store
- Clone gives you everything — no LFS, no API calls
- Content-addressed by SHA — git guarantees integrity

**Cons:**
- Repository size grows linearly with binary size × version count
- Slow clones for large registries
- GitHub/GitLab file size limits (100 MB hard limit per file)

---

### D — External content-addressed storage (S3, GCS, IPFS)

Binaries stored in an external CAS. The git tree holds a reference:

```toml
# packages/tool-a/package.toml
[package.binary]
url  = "s3://my-bucket/tool-a-1.0.0.tar.gz"
hash = "sha256:deadbeef..."
```

**Pros:**
- Unlimited size
- pgit verifies content hash on download — integrity guaranteed
- Storage survives git platform migration

**Cons:**
- Requires additional infrastructure
- More complex to set up for simple use cases
- Not "git-native"

---

## Recommendation by use case

| Use case | Recommended storage |
|----------|----------------------|
| Text/script packages (< 5 MB) | C — direct git objects |
| Binary distribution packages | B — release assets |
| Regulatory archive | B — release assets + hash in provenance.json |
| Very large binaries (> 100 MB) | D — external CAS (or A with caveats) |
| LFS | A — only for distribution registries, opt-in |

---

## Proposed pgit behaviour

- **Default**: no LFS. Text files committed directly. Binaries as release assets.
- **`--lfs` flag on `pgit init`**: opt-in for registries that need it (distribution only).
- **Archive mode**: LFS explicitly forbidden. Release assets + provenance hash required.
- **`pgit verify`**: re-downloads and re-hashes regardless of storage backend.

---

## Open Questions

1. Should pgit refuse to publish LFS-tracked files in an archive-mode registry, or
   warn and proceed?
2. For release assets: if the platform is self-hosted bare git (no release API),
   what is the fallback binary storage strategy?
3. Should pgit support option D (external CAS) in v1, or defer to v2?
4. Is there value in supporting IPFS or similar decentralised storage as an
   archival backend, given its content-addressing properties?
