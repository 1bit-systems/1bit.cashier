# 1bit.cashier

**A cash-only autonomous point-of-sale for small retail shops.**

No crypto. Real paper, real coins, local inference, no cloud on the hot path.

## What this is

1bit.cashier reduces the human cashier to a physical handoff role: they touch the product and hand it to the customer. Everything else — item recognition, order parsing, totaling, payment, change — is automated.

- **Vision** — cameras identify products as they cross the counter (zero-shot VLM in v0.1, fine-tuned YOLO in v1.0).
- **Audio** — boundary mic + `whisper.cpp` captures intent ("two of those," "the small one") and parses it against the current cart.
- **Cash** — a physical cash recycler (Glory CI-10 default) accepts notes and coins and dispenses exact change.
- **State** — a local event bus binds it all together. SQLite + filesystem for audit trail.
- **Display** — customer-facing screen shows the running order in real time. Bilingual: **English + Mi'kmaq (Listuguj orthography)**.

## Status

**Greenfield — pilot scope.** Designed for a single small retail shop on Mi'kma'ki (New Brunswick), with multi-location rollout deferred until pilot success.

| Release            | Target        | Scope                                                        |
| ------------------ | ------------- | ------------------------------------------------------------ |
| `v0.1.0-bench`     | weeks 1–3     | Runs on a fresh clone against mock hardware. No procurement. |
| `v0.2.0`           | weeks 4–6     | Real Glory CI-10 + cameras + mic installed in-shop.          |
| `v0.3.0`           | weeks 6–8     | Shadow mode in parallel with manual register.                |
| `v0.4.0`           | weeks 8–12    | Live evenings, AI-driven, cashier as override.               |
| `v1.0.0`           | Q4 2026       | Pilot success criteria hit, multi-location review.           |

## Design

Full architecture in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md). Canonical design spec in [`docs/specs/`](docs/specs/). Hardware bill-of-materials in [`docs/HARDWARE.md`](docs/HARDWARE.md). Deployment guide in [`docs/DEPLOY.md`](docs/DEPLOY.md).

## Roadmap and philosophy

1bit.cashier is the first module in a larger family covering accounting, taxes, and payroll — but not HR. The family follows a deliberate stance: **humans do the physical work, AI does the cognitive-administrative work, machines never talk to other machines.** Tooling integrates via Claude Code skills, not MCP servers.

Full family scope, sibling repos, and philosophy in [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Running the demo (planned, v0.1.0-bench)

```sh
git clone https://github.com/1bit-systems/1bit.cashier
cd 1bit.cashier
cargo run --bin cashier-pos -- --bench
# opens customer-display at http://localhost:8080
# mock vision/audio/recycler drive a scripted demo transaction
```

No hardware required to evaluate. Real shop data is wired in via env vars at deploy time (see [`docs/DEPLOY.md`](docs/DEPLOY.md)).

## License

[AGPL-3.0-only](LICENSE).

The AGPL choice is deliberate. Anyone running 1bit.cashier as a service or in a shop must publish their modifications. We want the autonomous-cashier ecosystem to stay open.

## Family

1bit.cashier is the first public project in the [1bit.systems](https://1bit.systems) autonomous-AI family. Other projects in that family are private and share no code with this repository — the family is brand, not codebase.
