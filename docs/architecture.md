# Architecture — openstack-tui (`os9s`)

_Last updated: 2026-09-24_

---

## High-Level Overview

`os9s` is a single-binary Rust TUI application structured around four main layers:

```
CLI / Boot (clap)
      │
      ▼
Config Layer (serde_yaml → clouds.yaml)
      │
      ▼
Auth Engine (Keystone v3 token, auto-refresh)
      │
      ▼
API Client Layer (reqwest async, per-service)
      │
      ▼
State Management (Arc<Mutex<AppState>>, background Tokio tasks)
      │
      ▼
TUI Rendering (Ratatui + crossterm, 250ms tick loop)
```

---

## Module Map

| Path | Purpose |
|---|---|
| `src/main.rs` | Entry point: Tokio runtime, auth bootstrap, poller spawn, TUI launch |
| `src/cli.rs` | `clap` arg definitions |
| `src/config/clouds.rs` | clouds.yaml parser (serde_yaml), resolution chain |
| `src/auth/keystone.rs` | Keystone v3 token auth, catalog parsing, re-auth |
| `src/client/mod.rs` | `OpenStackClient` (shared reqwest + auth header injection) |
| `src/client/nova.rs` | Hypervisors, servers, Nova services |
| `src/client/neutron.rs` | Networks, agents, routers |
| `src/client/swift.rs` | Swift /info + account stats (HEAD) |
| `src/client/cinder.rs` | Volumes, Cinder services |
| `src/client/glance.rs` | Image list |
| `src/state/mod.rs` | `AppState` struct + computed helpers |
| `src/state/poller.rs` | Background Tokio polling tasks + token refresher |
| `src/ui/app.rs` | Crossterm setup, main render/event loop, key handler |
| `src/ui/layout.rs` | Top-level layout compositor (tabs, split, status bar) |
| `src/ui/theme.rs` | Catppuccin-mocha color palette + Style helpers |
| `src/ui/views/overview.rs` | Cluster dashboard |
| `src/ui/views/nodes.rs` | Compute nodes view |
| `src/ui/views/servers.rs` | Running VMs view |
| `src/ui/views/networks.rs` | Network agents + network/router detail |
| `src/ui/views/swift.rs` | Swift stats view |
| `src/ui/views/volumes.rs` | Block volumes + Cinder services |
| `src/ui/views/images.rs` | Glance images |
| `src/ui/views/services.rs` | Nova + Neutron service health |

---

## State & Concurrency Model

```
Tokio Runtime (multi-thread, 4 workers)
│
├── nova_poller      (interval: 10s) ─────┐
├── neutron_poller   (interval: 15s) ─────┤
├── swift_poller     (interval: 30s) ─────┤── write → Arc<Mutex<AppState>>
├── cinder_poller    (interval: 20s) ─────┤
├── glance_poller    (interval: 60s) ─────┤
└── token_refresher  (check:    60s) ─────┘
                                           │
Main thread (250ms tick)                   │
  ├── try_lock AppState ──────── reads ────┘
  ├── render frame (Ratatui)
  └── poll crossterm events → mutate AppState
```

Pollers write data; the UI reads it. The Tokio Mutex ensures no torn reads.
The UI never blocks on a poller — it renders the last known state.

---

## Authentication Flow

1. Parse `clouds.yaml` → `CloudConfig`
2. POST `/v3/auth/tokens` with username+password+project scope
3. Extract `X-Subject-Token` header → `AuthToken.token`
4. Parse `token.catalog` → `ServiceCatalog` (service_type → interface → URL)
5. All API clients call `client.endpoint("compute")` etc. to resolve URLs
6. Token refresher task checks every 60s, re-auths if < 5 min remaining

---

## TUI Layout

```
┌─ Tabs (F1–F8) ──────────────────────────────────────────┐
│ F1 Overview  F2 Nodes  F3 VMs  F4 Networks ...          │
├─────────────────────────────┬───────────────────────────┤
│  LEFT PANE (55%)            │  RIGHT PANE (45%)         │
│  Scrollable resource list   │  Detail for selected row  │
│  (Table widget)             │  (Paragraph + scroll)     │
├─────────────────────────────┴───────────────────────────┤
│  STATUS BAR: keymap │ cluster health │ token TTL        │
└──────────────────────────────────────────────────────────┘
```

---

## Crate Selection Rationale

| Crate | Why |
|---|---|
| `ratatui` | Most actively maintained Rust TUI library, strong widget set |
| `crossterm` | Cross-platform raw mode, event stream, resize handling |
| `tokio` | Industry-standard async runtime; pairs perfectly with reqwest |
| `reqwest` (rustls) | TLS without OpenSSL C dep — pure Rust, smaller binary |
| `serde_yaml` | Standard clouds.yaml parsing |
| `fenix` (Nix) | Pinned Rust toolchain with rust-analyzer via Nix flakes |
| `crane` (Nix) | Correct incremental Cargo builds in Nix derivations |

---

## Planned Phase 4+ Additions

- Fuzzy search via `fuzzy-matcher` crate
- Column sorting (cycle with `s` key)
- `?` help popup modal
- `c` to copy UUID to clipboard via `arboard`
- Multi-cloud switching: `:cloud <name>` command mode
- Server console log viewer (`l` key)
- Placement API integration (resource providers, inventory)
