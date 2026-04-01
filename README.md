# pgit

Git-based package registry — use any GitHub or GitLab repository as a versioned package store.

Packages are identified by [PURL](https://github.com/package-url/purl-spec):

```
pkg:pgit/github.com/owner/repo/package-name@1.0.0
```

## Crates

| Crate | Description |
|---|---|
| `pgit-core` | Pure logic — PURL parsing, manifests, version resolution, hashing. WASM-safe. |
| `pgit-native` | I/O layer — git clone (with LFS), HTTP release assets, filesystem. |
| `pgit-wasm` | WASM build — wasm-bindgen exports and host I/O bridge. |
| `pgit` | CLI binary. |

## CLI

```
pgit add   github.com/owner/repo/name@1.0.0
pgit publish ./my-package --to github.com/owner/repo --version 1.0.0
pgit verify
pgit list  github.com/owner/registry
pgit archive github.com/owner/repo/name@1.0.0 --out ./snapshots
pgit init  --name my-registry [--archive] [--lfs]
```

## Registry modes

- **distribution** — versioned, yanking allowed.
- **archive** — append-only, immutable snapshots for regulatory/compliance use.

## Status

Early scaffold. Not yet ready for use.
