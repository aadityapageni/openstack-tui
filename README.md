# openstack-tui (`os9s`)

> **k9s for kolla-ansible OpenStack** — a blazing-fast terminal UI written in Rust 🦀

[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue)](LICENSE)
[![NixOS](https://img.shields.io/badge/Nix-flake-5277C3)](flake.nix)

---

## Features

- 🖥️  **Compute Nodes** — hypervisor state, vCPU/memory/disk utilization, running VMs
- ☁️  **Running VMs** — all-tenants server list with status, host, flavor breakdown
- 🌐  **Networks** — Neutron agent health, network list, router HA status
- 🗄️  **Swift** — object storage capacity, container/object counts
- 💽  **Volumes** — Cinder block volumes + service health
- 🖼️  **Images** — Glance image catalog
- 🔧  **Services** — Nova & Neutron service health at a glance
- 📊  **Overview** — cluster-wide dashboard
- 🔐  **clouds.yaml auth** — standard OpenStack credential format, admin token
- 🔄  **Auto token refresh** — re-auths 5 min before expiry
- ⌨️  **k9s-style keybindings** — F-keys, j/k, /, g/G, d/u

---

## Installation

### With Nix (recommended)

```bash
# Enter dev shell
nix develop

# Run directly
cargo run -- --cloud admin

# Or build + run via Nix
nix run .#openstack-tui -- --cloud admin
```

### From source

```bash
git clone https://github.com/aadityapageni/openstack-tui
cd openstack-tui
cargo build --release
./target/release/os9s --cloud admin
```

---

## Usage

```
os9s [OPTIONS]

Options:
  -f, --clouds <PATH>    Path to clouds.yaml [env: OS_CLIENT_CONFIG_FILE]
  -c, --cloud  <CLOUD>   Cloud name to use   [env: OS_CLOUD] [default: admin]
      --nova-interval <SECONDS>    [default: 10]
      --neutron-interval <SECONDS> [default: 15]
      --swift-interval <SECONDS>   [default: 30]
      --cinder-interval <SECONDS>  [default: 20]
  -v, --verbose          Verbose debug logging to ~/.local/share/os9s/os9s.log
  -h, --help             Print help
  -V, --version          Print version
```

### clouds.yaml example

```yaml
clouds:
  admin:
    auth:
      auth_url: https://keystone.example.com:5000
      username: admin
      password: your-admin-password
      project_name: admin
      user_domain_name: Default
      project_domain_name: Default
    region_name: RegionOne
    interface: internal
    identity_api_version: 3
```

Place at `~/.config/openstack/clouds.yaml` or pass via `--clouds`.

---

## Keybindings

| Key | Action |
|---|---|
| `F1` | Overview dashboard |
| `F2` | Compute nodes |
| `F3` | Running VMs |
| `F4` | Networks & agents |
| `F5` | Swift object storage |
| `F6` | Block volumes |
| `F7` | Glance images |
| `F8` | Service health |
| `↑/↓` or `j/k` | Navigate list |
| `/` | Inline search/filter |
| `Esc` | Clear search |
| `g` / `G` | Jump to top / bottom |
| `d` / `u` | Scroll detail pane down / up |
| `q` / `Ctrl+C` | Quit |

---

## Architecture

See [docs/architecture.md](docs/architecture.md) for the full system design.

---

## Development

```bash
# Enter dev shell
nix develop

# Hot-reload dev mode
cargo watch -x run -- --cloud admin

# Run tests
cargo nextest run

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Audit dependencies
cargo audit
```

Logs are written to `~/.local/share/os9s/os9s.log`.

---

## License

MIT
