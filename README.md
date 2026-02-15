# DietPi-Dashboard (CJackHwang Downstream)

A downstream fork of DietPi-Dashboard focused on UX improvements and ongoing maintenance.

## Upstream Relationship

- Upstream project: [nonnorm/DietPi-Dashboard](https://github.com/nonnorm/DietPi-Dashboard)
- This repository: [CJackHwang/DietPi-Dashboard-Next](https://github.com/CJackHwang/DietPi-Dashboard-Next)

## Installation

Use one of the precompiled releases, or compile from source.

### Release (Recommended)

This fork publishes two binaries: `frontend` and `backend`.

```sh
ARCH="${G_HW_ARCH_NAME:-x86_64}"
ASSET="dietpi-dashboard-${ARCH}.tar.gz"
URL="$(curl -sSf https://api.github.com/repos/CJackHwang/DietPi-Dashboard-Next/releases/latest \
  | mawk -F\" "/\"browser_download_url\": \".*dietpi-dashboard-${ARCH}\\.tar\\.gz\"/{print \\$4; exit}")"

curl -fL "$URL" -o "$ASSET"
tar -xzf "$ASSET"
chmod +x frontend backend
./frontend &
./backend
```

### Compile From Source

#### Prerequisites

```sh
dietpi-software install 9 16 17 # Node.js, Build-Essential, Git
corepack enable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo install just
```

#### Build and Run

```sh
git clone https://github.com/CJackHwang/DietPi-Dashboard-Next
cd DietPi-Dashboard-Next
cargo build --release -p frontend -p backend
./target/release/frontend &
./target/release/backend
```

## Open Dashboard

`http://<your-ip>:5252`

## CI / Release Automation

Workflow: `.github/workflows/push-build.yml`

- Push to `main`: runs lint + cross builds and uploads CI artifacts.
- Publish a GitHub Release: automatically cross-builds all targets and uploads release assets (`.tar.gz` + `.sha256`) to that release.
