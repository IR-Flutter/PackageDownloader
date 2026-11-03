# PackageDownloader 2.0 — Installation Guide

PackageDownloader (`pdown`) is distributed as a single standalone executable binary with zero external runtime dependencies. Neither Dart, Flutter, Java, nor Rust is required on the user machine to run the compiled tool.

---

## 1. Supported Platforms & Binaries

| Operating System | Architecture | Artifact Name |
| :--- | :--- | :--- |
| **macOS Apple Silicon** | `aarch64-apple-darwin` (M1–M4) | `pdown-macos-arm64` |
| **macOS Intel** | `x86_64-apple-darwin` | `pdown-macos-x64` |
| **Linux x64** | `x86_64-unknown-linux-gnu` | `pdown-linux-x64` |
| **Windows x64** | `x86_64-pc-windows-msvc` | `pdown-windows-x64.exe` |

---

## 2. One-Line Installation

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/install.ps1 | iex
```

The installer automatically detects OS and architecture, verifies SHA-256 checksums from the signed release manifest, and places the binary into your PATH.

---

## 3. Manual Installation from GitHub Releases

1. Visit [GitHub Releases](https://github.com/IR-Flutter/PackageDownloader/releases).
2. Download the binary matching your platform along with `checksums.txt`.
3. Verify SHA-256:
   ```bash
   sha256sum -c checksums.txt --ignore-missing
   ```
4. Move binary to your user PATH (e.g. `/usr/local/bin/pdown` or `~/.local/bin/pdown`).
5. Ensure executable permissions:
   ```bash
   chmod +x /usr/local/bin/pdown
   ```
6. (Optional) Create `irflutter` alias:
   ```bash
   ln -s /usr/local/bin/pdown /usr/local/bin/irflutter
   ```

---

## 4. Building from Source

Requirements: Stable Rust (1.75+).

```bash
git clone https://github.com/IR-Flutter/PackageDownloader.git
cd PackageDownloader
cargo build --release
./target/release/pdown --version
```
