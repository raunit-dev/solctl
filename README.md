# solctl – Minimal Solana CLI Tool

A lightweight Solana CLI written in Rust using `clap` and the Solana RPC client, made for interacting with the Solana **devnet**.  
Supports SOL transfers, airdrops, and balance checks – with default wallet fallback from `~/.config/solana/id.json`.

> Built out of boredom. Contributions welcome!

---

##  Features

- Check wallet balance
- Airdrop SOL (devnet only)
- Transfer SOL (uses default keypair if not provided)
- Uses RPC with commitment level `confirmed`

---

## 📦 Installation

```bash
git clone https://github.com/raunit-dev/solctl.git
cd solctl
cargo build --release
