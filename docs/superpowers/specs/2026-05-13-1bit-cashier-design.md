# 1bit.cashier — Design Spec

| Field          | Value                                          |
| -------------- | ---------------------------------------------- |
| Project        | 1bit.cashier                                   |
| Codename retired | `autotill` (handoff placeholder)             |
| Date           | 2026-05-13                                     |
| Status         | Approved for implementation                    |
| License        | AGPL-3.0-only                                  |
| Repository     | `1bit-systems/1bit.cashier` (public)           |
| Family         | First public project in the 1bit.systems autonomous-AI family |
| Source         | Adapted from `~/Documents/HANDOFF.md` (2026-05-13) with deviations noted in §13 |

---

## 1. Summary

1bit.cashier is a cash-only, AI-mediated point-of-sale for a single small retail shop on Mi'kma'ki (New Brunswick). The human cashier is reduced to a physical handoff role: they touch the product, slide it across a glass counter, and hand it to the customer. **Everything else — item recognition, order parsing, totaling, payment, change — is automated.** The customer pays cash into a recycler that dispenses exact change. The entire hot path runs locally on a Strix Halo (Ryzen AI MAX+ 395) workstation; no cloud dependency.

The pilot is one shop, evening operation. If success criteria (§12) are met after 60 days, the project generalizes for multi-location rollout.

## 2. Goals and non-goals

### Goals
- Replace the cashier's active POS role with vision + audio + a cash recycler.
- Local-first: nothing on the hot path requires cloud access.
- Recoverable: every transaction reconstructible from on-disk audit data.
- Open source from day one (AGPL-3.0).
- Bilingual customer-facing surface: **English + Mi'kmaq (Listuguj orthography)**. No French.

### Non-goals (for the pilot)
- No card / contactless / mobile payment. **Cash only.**
- No customer self-checkout. There's still a person behind the counter.
- No cloud on the hot path. Local inference on the Strix Halo.
- No SKU database overhaul. The existing shop inventory list is the starting catalog.
- **No cryptocurrency.** The `.cashier` name is a deliberate distancing from the `.cash` TLD's crypto associations.

## 3. Architecture (single-box, event-driven)

```
                ┌──────────────────────────────────────────────────┐
                │              Strix Halo (Ryzen AI MAX+)          │
                │                                                  │
  [cam 1: glass]──► vision-svc ──┐                                 │
  [cam 2: counter]──►            │                                 │
                                 ▼                                 │
  [boundary mic] ──► audio-svc ──► order-state ──► customer-display│
                                 ▲          │                      │
                                 │          ▼                      │
                          cashier-tablet   audit-log (sqlite)      │
                                 ▲          │                      │
                                 │          ▼                      │
                      cash-recycler-svc ───►nightly-sync           │
                                 │                                 │
                └────────────────┼─────────────────────────────────┘
                                 │
                          [Glory CI-10 / equiv]
                          (serial or TCP API)
```

Every service publishes events over an in-process event bus. `order-state` is the single source of truth for the current transaction. The audit log captures everything: vision frames at moment-of-recognition, transcripts, cash events, final receipt, and a short video clip of the handoff zone.

## 4. Subsystems

### 4.1 `cashier-vision`
**Purpose:** identify products as they cross the handoff zone on the glass counter.

**Approach (phase 1, fastest path):** zero-shot VLM matching against a product catalog of reference images. Use Qwen2.5-VL-7B or InternVL2 via the local inference stack (lemonade / T-MAC where supported). Trigger on motion in a defined polygon on the counter surface; capture frame, run identification, emit event.

**Approach (phase 2, once we have data):** fine-tuned YOLOv11 detector on actual SKU photos collected during phase 1. Falls back to VLM for unknowns.

**Why this order:** SKU list will churn during the pilot. Retraining YOLO every time someone adds a new candy bar is operationally hostile. VLM with a JSON catalog is editable in 30 seconds.

**Camera spec:** USB3 or PoE IP cam, 1080p minimum, fixed focus on the handoff zone, ~60cm above counter, matte black non-reflective mat as the recognition surface. Diffuse overhead LED, no spotlights (kills glare on packaging).

**Output event:**
```json
{ "ts": "...", "sku": "string|null", "confidence": 0.92,
  "bbox": [x,y,w,h], "frame_uri": "audit://...", "candidates": [...] }
```

### 4.2 `cashier-audio`
**Purpose:** capture customer intent the camera can't see ("two of those," "the small one," "actually make it three").

**Pipeline:** boundary mic over the counter → VAD-gated chunks → `whisper.cpp` (large-v3-turbo) → small local instruct model as intent parser, with the running cart + product catalog in context.

**Intent parser prompt skeleton:**
```
You are a POS order parser. Given the running cart and a transcript,
emit JSON actions: add/remove/change_qty/void/clarify.
Cart: {...}
Catalog: {...}
Transcript: "..."
Output: { "actions": [...] }
```

**Fallback:** push-to-talk button on the cashier tablet if ambient noise breaks VAD. Phase 1 ships with PTT as the default and VAD as opt-in once the shop's noise profile is characterized.

**Language scope (v0.1–v0.4):** audio input is **English-only**. See §6 for Mi'kmaq handling. Add a `language: "en"` field to the audio event schema from day one so future Mi'kmaq STT can land without redesign.

### 4.3 `cashier-order`
**Purpose:** authoritative cart for the current transaction. Subscribes to vision and audio events, applies them, broadcasts state to the customer display and cashier tablet.

**Rule:** vision events are source of truth for *what's on the counter*. Audio events add qualifiers, quantities, and corrections. Conflicts (camera sees one bag, customer says "two") → cashier confirmation required on the tablet.

**State machine:** `idle → building → review → awaiting-payment → paying → complete → idle`. Stuck states auto-page the cashier tablet after configurable timeout.

This crate is the spine. Build it first; everything else attaches to it.

### 4.4 `cashier-recycler`
**Purpose:** thin adapter over the physical cash machine.

**Recommended hardware:**
- **Glory CI-10** (or CI-5 for tighter footprint) — gold standard, Canadian-currency support, well-documented CashInfinity API.
- **SUZOHAPP Eagle / Bulk Coin Recycler** as a cheaper alternative.
- **Volumatic CounterCache intelligent** if budget is tight and we only need note handling (no coin recycling).

**Lead time is the long pole** — 4 to 8 weeks typical, sometimes more for Canadian deployments. Order on day one.

**API surface:**
```
charge(amount_cents) -> { tendered, change_dispensed, status }
status() -> { cassette_levels, jams, doors_open, ... }
end_of_day_report() -> { totals, denominations, ... }
```

**Mock backend:** ships in-crate behind a feature flag, used by `cashier-pos --bench`. Returns scripted responses so the bench MVP can run with no hardware.

**Do NOT DIY** with bare bill validators (MEI, JCM) and coin hoppers. Compliance, security, and reliability are not worth the build.

### 4.5 `cashier-audit`
**Purpose:** every transaction reconstructible from disk.

For each transaction store: line items, vision frame(s) at moment of recognition, audio transcript, cash recycler event log, final receipt PDF (or text), short video clip (5s before + 5s after) of the handoff zone. SQLite + filesystem. Nightly rsync/restic to a NAS or off-box backup.

Also the **training data pipeline** for phase 2 (YOLO fine-tune) and for the multi-location rollout. **Opt-in audio retention** is the seed for a future Mi'kmaq STT corpus (see §6).

### 4.6 `cashier-bus` — event bus
**Purpose:** event distribution between services.

**Design:** `EventBus` trait. Default implementation is `tokio::sync::broadcast` (in-process, single-box). Optional NATS backend behind a `nats` feature flag for future multi-host distribution.

**This deviates from the handoff** which specified "embedded NATS." See §13.D1 for rationale.

### 4.7 `apps/cashier-display` — customer display
**Purpose:** show the customer what's being added to their order in real time, then the total, then payment instructions. **Bilingual: English + Mi'kmaq (Listuguj).**

**Stack:** PWA, single HTML/JS file, websocket to `cashier-order`. Big text, high contrast, accessible for older customers and low-light conditions. Reuse the 1bit.systems palette (`#0d1117` background, `#00d4ff` accent, monospace) for family visual continuity.

**Hardware:** 15–22" monitor on a swivel stand, customer-facing side of the counter. Capacitive touch optional but not required for phase 1.

### 4.8 `apps/cashier-pos` — main binary
**Purpose:** wire all services together, host the websocket server for the display, expose the bench / production switch.

Flags:
- `--bench` — uses mock vision/audio/recycler, drives a scripted demo transaction, no hardware required.
- `--real` — uses real hardware adapters. Requires env vars (see §7).

### 4.9 `cashier-tablet` (route under `apps/cashier-display`)
**Purpose:** override and confirmation surface for the human. Approves ambiguous detections, handles voids, calls for help, drives PTT.

**Stack:** same PWA, different route. Cheap Android tablet on a stand near the till.

## 5. Tech stack

| Layer              | Choice                                         | Rationale                                                |
| ------------------ | ---------------------------------------------- | -------------------------------------------------------- |
| OS                 | CachyOS                                        | Existing daily driver                                    |
| Service language   | Rust (edition 2024)                            | Single-box single-binary; aligns with 1bit.systems convention |
| Vision inference   | T-MAC / lemonade runtime                       | Local, ternary-friendly where supported                  |
| STT                | `whisper.cpp` large-v3-turbo                   | Battle-tested on Halo                                    |
| Intent LLM         | Small local instruct model (bitsy class)       | Same stack, no cloud                                     |
| Event bus          | `tokio::sync::broadcast` (default) + NATS feat | YAGNI: in-process for single-box, NATS when distributed  |
| Store              | SQLite + filesystem                            | Boring, recoverable                                      |
| Customer display   | Vanilla HTML / JS PWA                          | No framework tax for a single screen                     |
| Cash recycler      | Glory CI-10 over TCP                           | Buy not build                                            |

**No Python on the hot path.** Python permitted only in `/training/` (offline YOLO fine-tune) and `/scripts/` (operator tooling).

## 6. Localization

### Locales shipped
- `en` (default)
- `mikq-LJ` — Mi'kmaq, Listuguj orthography (locally appropriate for NB / Gaspé)
- `mikq-SF` — Mi'kmaq, Smith-Francis (post-v0.1 if cross-community deployment lands)

**Explicitly excluded: French.** This is a deliberate Indigenous-first choice for a deployment on Mi'kma'ki, replacing the default NB English/French bilingual posture.

### Output scope (all bilingual switchable)
- Customer-display PWA
- Cashier-tablet UI
- Receipt format (printed and digital)
- Static signage assets (if produced)

### Input scope (v0.1–v0.4)
**Audio input is English-only.** Whisper-v3-turbo has no usable Mi'kmaq support, and small intent-parser LLMs don't speak it. Mi'kmaq is polysynthetic (`Setawi'gwug` ≈ "I want to buy it" — one word, not five), which breaks English-trained tokenizer assumptions even if STT existed.

Mi'kmaq-speaking customers get the **cashier-tablet PTT** path: cashier enters items manually, but display + receipt are presented in Mi'kmaq.

### Future direction (out of pilot scope)
A Mi'kmaq STT corpus could be seeded from `cashier-audit`'s opt-in audio retention, in partnership with a Mi'kmaq language organization. The audit-log opt-in must be designed so this becomes possible later without redesign — that's the only architectural commitment now.

### Translation workflow
Locale files at `apps/cashier-display/locales/{en,mikq-LJ}.json`. Translation contributions are public, AGPL-licensed alongside the rest of the project. CONTRIBUTING.md will document review by a Mi'kmaq community contact before locale changes merge.

### Font
Mi'kmaq Listuguj uses Latin letters + apostrophe + acute accents — covered by most modern fonts. Display defaults to a font with good Latin Extended-A coverage (Inter / Source Sans / system stack).

## 7. Repo layout

```
1bit.cashier/
├── crates/
│   ├── cashier-bus/          # EventBus trait, tokio impl, optional NATS feature
│   ├── cashier-vision/
│   ├── cashier-audio/
│   ├── cashier-order/
│   ├── cashier-recycler/     # Glory CI-10 adapter + mock
│   └── cashier-audit/
├── apps/
│   ├── cashier-pos/          # main binary; --bench / --real
│   └── cashier-display/      # PWA static assets; locales/{en,mikq-LJ}.json
├── example/
│   ├── catalog/products.json # fake-SKU demo catalog
│   └── ref-images/           # CC-licensed reference photos for VLM matching
├── training/                 # Python-only, offline (YOLO fine-tune)
├── deploy/
│   └── systemd/              # unit files per service
├── docs/
│   ├── ARCHITECTURE.md
│   ├── HARDWARE.md
│   ├── DEPLOY.md
│   └── superpowers/specs/    # design specs (this file lives here)
├── Cargo.toml                # workspace
├── README.md
├── LICENSE                   # AGPL-3.0
└── .gitignore
```

### Env vars (production)
| Var                       | Purpose                                                |
| ------------------------- | ------------------------------------------------------ |
| `CASHIER_CATALOG_PATH`    | Real-shop catalog (overrides `example/catalog`)        |
| `CASHIER_REF_IMAGES_DIR`  | Real ref-images directory (overrides `example/ref-images`) |
| `CASHIER_RECYCLER_HOST`   | Glory CI-10 TCP host:port                              |
| `CASHIER_AUDIT_DIR`       | Audit log root (default `/var/lib/cashier/audit`)      |
| `CASHIER_BACKUP_TARGET`   | rsync target for nightly sync                          |
| `CASHIER_LOCALE_DEFAULT`  | Default locale (`en` or `mikq-LJ`)                     |

## 8. Phase plan / milestones

### Phase 0 — Procurement (week 0)
- Spec & order cash recycler (4–8 week lead time, start NOW)
- Order cameras, mics, monitor, tablet, mount hardware
- Inventory walk: photograph every SKU in the shop, log to a **private** catalog (NOT committed)

### Phase 1 → `v0.1.0-bench` (weeks 1–3) — **Public release tagged**
- Workspace skeleton: crates, apps, example dataset
- `cashier-bus` with `EventBus` trait + tokio impl + unit tests
- Event schemas (`VisionDetection`, `AudioIntent`, `OrderUpdate`, `CashEvent`, `AuditWrite`), every event carries `ts`, `transaction_id`, `correlation_id`, `language`
- `cashier-order` state machine with exhaustive unit tests
- Mocks for vision / audio / recycler (driven by `cashier-pos --bench`)
- Customer-display PWA wired to `cashier-order` via websocket; English locale shipping, Mi'kmaq-LJ stubs
- README, ARCHITECTURE.md, HARDWARE.md, DEPLOY.md filled in
- `cargo run --bin cashier-pos -- --bench` produces a watchable scripted demo transaction with no hardware

### Phase 2 → `v0.2.0` (weeks 4–6, gated on procurement)
- Physical install: cameras, lighting, mics, monitor, tablet, cash machine
- Real `cashier-recycler` against Glory hardware
- Cashier-tablet override flows
- `cashier-audit` writing real artifacts to disk; nightly backup operational
- Mi'kmaq-LJ locale complete (after community review per §6)

### Phase 3 → `v0.3.0` (weeks 6–8) — Shadow mode
- System runs in parallel with the manual register
- Cashier still operates normally; system records what *it* would have done
- Nightly discrepancy report: AI vs. human
- Fix detection failures, expand catalog, tune intent parser

### Phase 4 → `v0.4.0` (weeks 8–12) — Live evenings
- Register switches to AI-driven during evening shift, cashier as override only
- Daily reconciliation: system totals vs. cash cassettes
- Audio retention opt-in collects data for Mi'kmaq STT corpus
- Collect data for YOLO fine-tune

### Phase 5 → `v1.0.0` (Q4 2026) — Pre-rollout hardening
- YOLO model trained on real shop data, deployed alongside VLM fallback
- Multi-location architecture review (what changes when there's N shops?)
- Remote management plane design — likely the trigger for enabling the NATS bus feature

## 9. Open questions (gate Phase 0–2, not v0.1.0-bench)

These shape the deployment and should be answered before Phase 0 procurement completes. They do **not** block the bench MVP.

1. **SKU count.** Roughly how many distinct products? Determines whether VLM zero-shot is viable as a long-term plan or just a bootstrap.
2. **Visually similar variants.** Near-identical packaging at different prices (flavor / size variants)? These force barcode fallback or audio disambiguation.
3. **Age-restricted products.** Anything legally requiring ID check? Forces cashier present-and-active, which changes the unattended-evening story.
4. **NB regulatory.** Unattended cash transaction rules, receipt requirements, weights-and-measures certification for the cash machine. Half-day of research before procurement.
5. **Power & network.** Shop on UPS? Internet drop behavior? System should keep working local-first; nightly sync must recover gracefully.
6. **Loss prevention.** Who reviews audit footage when totals don't reconcile? Cashier? Owner? Insurance requirement?
7. **Mi'kmaq community contact.** Who reviews `mikq-LJ` locale strings before they merge? Identify before Phase 2.

## 10. Failure modes (predictions; plan accordingly)

- **Glare on the counter glass** — VLM hallucinates SKUs from reflections. Test lighting day one.
- **Whisper hallucinations on silence** — VAD threshold per environment; revisit weekly during Phase 3.
- **Cash recycler jams** — they happen. Cashier-tablet needs a "clear jam" workflow that pauses transactions, not a hard error.
- **Customer cancels mid-order** — `cashier-order` needs a clean abort path that doesn't poison the audit log or leave the cash machine in a weird state.
- **Two customers at once** — Phase 1 assumes one transaction at a time. Document for the cashier; don't multi-track until Phase 4 at earliest.
- **Power loss mid-transaction** — SQLite WAL + recycler's own transactional state should cover this; write the recovery procedure before it's needed.
- **Mi'kmaq locale fallback** — if a string isn't translated, fall back to English silently, log a missing-string warning. Never display `__missing_locale_key__` to a customer.

## 11. First tasks (rough priority order)

This list is what `writing-plans` will turn into an executable plan. Don't proceed past task 3 until Phase 0 procurement is in motion.

1. **Scaffold crates.** Create empty crate skeletons under `crates/` matching §7. Register members in workspace `Cargo.toml`. `lib.rs` + `README.md` per crate.
2. **Define event schemas** in `cashier-bus`. Rust enums with `serde`. Every event carries `ts`, `transaction_id`, `correlation_id`, `language`.
3. **Build `cashier-order` first.** Full state machine, all transitions, exhaustive tests. Easiest to unit-test end-to-end without hardware. This is the spine.
4. **Mock services.** Mock `cashier-vision` (fires events from a directory of test images on a timer), mock `cashier-audio` (scripted transcripts), mock `cashier-recycler` (simulates Glory). Get the whole loop running on bench, no hardware.
5. **Customer-display PWA.** Single HTML file, websocket client, renders order state. Use the 1bit.systems palette. English locale only initially; Mi'kmaq-LJ stubs in place. Keep under 500 lines.
6. **Tag `v0.1.0-bench`** and publish the release on GitHub.
7. *(Phase 2)* **Real `cashier-vision`.** Webcam capture (`nokhwa` or `v4l2`), motion trigger, frame capture, local VLM endpoint, parse response, emit event.
8. *(Phase 2)* **Real `cashier-audio`.** Mic capture (`cpal`), VAD (Silero or webrtc-vad), chunk to `whisper.cpp`, transcript to intent parser, emit event.
9. *(Phase 2)* **Real `cashier-recycler`.** Against Glory's API spec once the unit arrives. Until then, mock is canonical.

## 12. Success criteria

The pilot is a success if, after 60 days of evening operation:

- ≥98% of transactions complete without cashier override on totals
- Zero cash discrepancies at end-of-day cassette reconciliation traced to system error (vs. human counting error)
- <5 second median time from "last item on counter" to "total displayed to customer"
- Audit log is complete and queryable for every transaction
- Cashier reports the job is *easier*, not harder, than the manual register
- Mi'kmaq-speaking customers complete transactions with no degraded UX vs. English-speaking customers (display, receipt, and PTT flow all functional)

If those hit, we're cleared to design the multi-location rollout (Phase 5).

## 13. Deltas from source handoff

This spec is adapted from `~/Documents/HANDOFF.md` (2026-05-13). Deltas, with rationale:

### D1. Event bus: `tokio::sync::broadcast` by default; NATS behind a feature flag.
Handoff specified embedded NATS. Embedded NATS in Rust is awkward — no first-class embedded server crate, and forcing contributors to install `nats-server` before they can `cargo run` the demo is a real onramp tax for a public AGPL project. In-process channels are simpler to test, debug, and ship for a single-box pilot. Trait-shaped from day one means swapping to NATS later costs hours, not days. NATS becomes meaningful when multi-host or remote management plane arrives, which is Phase 5+ territory.

### D2. Crate naming: `cashier-*`, not `autotill-*`.
Project rename from the handoff's placeholder codename. All crates, the top-level binary (`cashier-pos`), and the display app (`cashier-display`) use the `cashier-` prefix.

### D3. Example dataset committed to the repo.
Handoff was silent on this; the public-AGPL posture makes it explicit: ship `example/catalog/` (fake SKUs) and `example/ref-images/` (CC-licensed photos) so `cargo run --bin cashier-pos -- --bench` works on a fresh clone with no setup. Real shop data is env-override only and lives outside the repo.

### D4. No-Python-on-hot-path policy carried over.
Aligns 1bit.cashier with 1bit.systems convention. Python permitted only in `/training/` (offline YOLO fine-tune) and `/scripts/` (operator tooling).

### D5. `v0.1.0-bench` is a public-tagged release goal.
Handoff's Phase 1 was a "bench MVP" without a release framing. For a public project, the first taggable artifact that anyone can clone and run is a load-bearing milestone — it's the visible product before any hardware is procured.

### D6. `docs/` directory structure.
Handoff didn't address documentation layout. Public project demands more than a README: `docs/ARCHITECTURE.md` (evergreen), `docs/HARDWARE.md` (BOM), `docs/DEPLOY.md` (env vars / systemd), `docs/superpowers/specs/` (canonical design specs including this one).

### D7. Localization is a first-class section (new §6).
Handoff was silent on language. NB defaults to English/French bilingual; this project explicitly chooses **English + Mi'kmaq (Listuguj)**, no French. Audio input remains English-only for v0.1–v0.4 because Mi'kmaq STT doesn't yet exist. Display/receipt are bilingual switchable from v0.1.

### D8. Phase milestones get semver tags.
Handoff phases were time-bound. Adding semver tags (`v0.1.0-bench` → `v1.0.0`) gives the public repo a release rhythm independent of calendar weeks.

## 14. References

- **Source handoff:** `~/Documents/HANDOFF.md` (2026-05-13). Authoritative for subsystem responsibilities, hardware choices, failure modes, and success criteria. This spec extends and adapts it; deviations listed in §13.
- **Sibling family:** [1bit.systems](https://1bit.systems) — brand parent, no shared codebase.
- **Glory CashInfinity API** — buy/spec the CI-10 unit; lead time is the long pole, order day one.
- **Mi'kmaq Listuguj orthography** — local language standard for NB / Gaspé Mi'kmaq communities.
- **AGPL-3.0** — license text at `LICENSE`; explicit choice to prevent proprietary forks of autonomous-cashier infrastructure.

---

*End of spec. Updates land as a new dated file in `docs/superpowers/specs/`; this one stays as the v0.1.0 baseline.*
