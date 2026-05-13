# 1bit.cashier — Roadmap

This roadmap covers two layers: the **family** of related projects that 1bit.cashier sits inside, and the **philosophy** that explains why the family is shaped this way. For the milestone plan of *this* repo specifically (`v0.1.0-bench` → `v1.0.0`), see [the design spec, §8](specs/2026-05-13-1bit-cashier-design.md#8-phase-plan--milestones).

---

## 1. Family scope

1bit.cashier is the first project in the [1bit.systems](https://1bit.systems) autonomous-AI family. The family's scope is **the full administrative back-office for a small business, minus HR.** Each module is a separate AGPL-3.0 sibling repo under `1bit-systems/`, sharing brand and philosophy but no code.

| Repo                          | Scope                                                       | Status                |
| ----------------------------- | ----------------------------------------------------------- | --------------------- |
| `1bit-systems/1bit.cashier`   | Point-of-sale, cash-only, AI-mediated                       | **Active (this repo)** |
| `1bit-systems/1bit.ledger`    | General ledger, AP, AR, reconciliation, journal entries     | Planned               |
| `1bit-systems/1bit.tax`       | Sales tax (GST / HST / PST), income tax, payroll-tax filings | Planned               |
| `1bit-systems/1bit.payroll`   | Employee compensation, deductions, T4s (Canada)             | Planned               |

**HR is explicitly out of scope** for the family. Rationale in §2.

---

## 2. Philosophy

### 2.1 Humans do the physical. AI does the cognitive-administrative.

1bit.cashier inverts the typical "agentic commerce" stance. Most current AI infrastructure tries to remove the human from transactions: machine-to-machine payments, agent-to-agent commerce, autonomous shopping bots. We reject that.

The right division of labor for a small shop:

- **Humans** stock shelves, walk product to the counter, hand it to customers, clean, repair, restock, deal with people, and own the shop. The dignity of physical work stays with humans.
- **AI / automation** does the cognitive overhead: item recognition, totaling, change calculation, audit trails, ledgers, tax filings, payroll. These tasks bore humans, take time, and produce errors.

We do not connect machines to other machines. We connect humans to AI, and let humans connect to other humans.

### 2.2 No HR module — by design

HR exists in larger companies because there's structural misalignment between management and labor. In a shop where everyone is doing real work (including the owner), HR's bureaucratic functions — performance reviews, complaint mediation, compliance training, hiring funnels — don't apply.

Payroll, yes — but payroll is accounting, not HR.

### 2.3 No M2M (machine-to-machine) automation

No agentic-commerce rails. No x402 micropayments. No automated procurement bots. No supplier-to-cashier API integrations.

**All inputs to the system originate from a human in the shop.** Cameras see what humans put on the counter. Microphones hear what humans say. The cash recycler processes physical bills and coins from human hands. Audit logs export to flat files that humans hand off to downstream modules.

### 2.4 Skills, not MCP servers

The integration model for AI tooling across the family is **Claude Code skills** (markdown skill files), not MCP servers exposing tools.

Reasoning: skills are operational know-how — *"how to close out the books for the day"*, *"how to file Q1 GST/HST returns"*, *"how to reconcile this week's cassette pulls"* — versioned in the repo, reviewable in PRs, executable only with a human's explicit ask. MCP servers are machine-to-machine protocols by design; they fit the M2M model we reject.

A shop owner runs Claude Code locally. Skills know how to drive the cashier CLI, query the SQLite audit log, generate reports, file taxes. **Humans-in-the-loop at every step.**

Skills land in a sibling repo (`1bit-systems/1bit.skills` or equivalent), not in this one.

### 2.5 The "Human 3.0" stance — go full measures or it doesn't work

External thinkers on human-AI augmentation frame the current transition as "Human 3.0" or similar (citation pending — see [open question](#5-open-roadmap-questions)). The position taken here is: this transition only works if you commit fully. Half-measures that try to preserve the worst parts of office work — HR bureaucracy, machine-to-machine procurement, autonomous agent commerce, paperwork theater — won't reach 3.0. They just churn 2.0 with more friction and more API calls.

Full measures: **humans physical, AI administrative, no M2M, no HR.**

---

## 3. How philosophy affects this repo specifically

- **No MCP server inside 1bit.cashier.** `cashier-pos` exposes a websocket for the customer display and a CLI for operators. No tool-server posture.
- **The audit log is the integration surface** for downstream modules (`1bit.ledger`, `1bit.tax`). Cash events, transaction line items, recycler reports — all exportable as flat files or queryable via SQLite. No live RPC.
- **No live API to any cloud accounting service** (QuickBooks, Wave, Xero). Books are kept locally. Export-on-demand only.
- **Skills target the operator, not the shop.** Future `1bit-systems/1bit.skills` repo provides skills like *close-out-day*, *file-q1-gst-hst*, *reconcile-cassettes* — all run by a human via Claude Code, not by an autonomous agent.

---

## 4. Family timeline (subject to revision as the pilot teaches us)

| Quarter   | Milestone                                                            |
| --------- | -------------------------------------------------------------------- |
| Q2 2026   | 1bit.cashier `v0.1.0-bench` (public, mock hardware) → `v0.4.0` (pilot evenings live) |
| Q3 2026   | 1bit.cashier `v1.0.0`; multi-location review opens                   |
| Q4 2026   | 1bit.ledger `v0.1` — general ledger fed by 1bit.cashier audit exports |
| Q1 2027   | 1bit.tax `v0.1` — Canadian GST / HST quarterly filing path           |
| Q2 2027   | 1bit.payroll `v0.1` — T4 cycle for the pilot shop's employees        |
| H2 2027   | Family `v1.0` integrated rollout to a multi-location operator        |

---

## 5. Open roadmap questions

These are tracked here, not in the main spec, because they shape the *family* posture rather than `v0.1.0-bench` specifically.

1. **"Human 3.0" attribution.** Which thinker(s) get cited for the framing in §2.5? Pending confirmation before any external name lands in a public commit.
2. **`1bit.skills` repo placement.** Sibling repo (`1bit-systems/1bit.skills`) or a subdirectory in this one? Subdirectory keeps everything together; sibling lets skills evolve independent of cashier releases. Decide before the first skill ships.
3. **Audit-log schema as a public contract.** If `1bit.ledger` consumes 1bit.cashier exports, the export format becomes a versioned contract. Worth a schema-versioning section in `docs/DEPLOY.md` before `v0.4.0`.
4. **Tax surface for non-NB / non-Canada deployments.** Pilot is NB. If anyone forks `1bit.cashier` outside Canada, the GST/HST/PST assumptions need pluggable tax modules. Defer until a non-Canadian deployment is real.

---

*This roadmap is part of the public AGPL repo. Updates are merged via PR like any other doc.*
