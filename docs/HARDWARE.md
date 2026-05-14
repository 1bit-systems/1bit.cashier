# Hardware

Procurement guide, bill-of-materials, and installation notes for the physical components of a 1bit.cashier deployment.

> **Status: in progress.** Phase 0 procurement decisions land here as they're made. None of this is required to evaluate v0.1.0-bench — `cargo run -p cashier-pos -- --bench` uses mocks for every hardware boundary. Real hardware integrates in v0.2.0.

## Vision — cameras + PoE infrastructure

The vision subsystem (`crates/cashier-vision`) recognizes products as they cross the handoff zone. In v0.2+ it pulls frames from PoE IP cameras on motion trigger. **Motion-triggered single-frame grab tolerates 200–500ms RTSP latency** — industrial-grade <50ms cameras are not required for v0.1–v0.4 and represent a 5–10× cost increase for marginal pilot benefit.

### Three-camera layout

| Cam | Position                            | Job                                              | FOV / lens   |
| --- | ----------------------------------- | ------------------------------------------------ | ------------ |
| A   | Top-down, ~60cm above counter       | Primary SKU recognition on the matte mat         | 95° / 2.8mm  |
| B   | 45° angle from above, side-mount    | 3D context (cylinders, multi-packs, lids)        | 75° / 4mm    |
| C   | Customer-facing, wide, lower res    | Audit-log context clip (5s before/after handoff) | 110° / 2.0mm |

Cams A + B together cover the failure modes of pure top-down recognition (look-alike packaging from above). Cam C is for the audit log per design spec §4.5.

### Procurement tiers

| Tier                       | Cameras (×3)                                | Switch                             | Approx total CAD | RTSP latency | When to pick                              |
| -------------------------- | ------------------------------------------- | ---------------------------------- | ---------------- | ------------ | ----------------------------------------- |
| **Pilot (recommended)**    | **Reolink RLC-820A** 4K PoE                 | TP-Link **TL-SG108PE** 8-port PoE+ | ~$340            | 300–500 ms   | v0.1–v0.4 evening pilot                   |
| Mid-tier                   | **Hikvision DS-2CD2143G0** 4MP PoE + SDK    | same switch                        | ~$700            | ~200 ms      | If pilot exposes consumer-tier limits     |
| Industrial                 | **Basler daA1280-54uc** USB3 / dart GigE    | managed Gigabit switch             | $2k–$4k+         | <50 ms       | v1.0 multi-location rollout               |

### Notes

- **PoE + hardwired only.** No WiFi cameras. PoE+ (802.3at, 30W/port) is required for 4K consumer cams; the TL-SG108PE provides 4 PoE+ ports + 4 plain PoE ports.
- **Cabling.** Cat 5e for runs under 30m; Cat 6 for longer. PoE+ runs 100m max at full power. Buy pre-made patch cables from Monoprice/Amazon to skip crimping.
- **Reolink SKU caveat.** Only the `-A` suffix (PoE). Avoid the WiFi-only variants.
- **Mounting.** Ceiling-mount junction boxes + adjustable arms for cams A and B; small wall mount for cam C.
- **Software path (v0.2+).** `cashier-vision` will use the `nokhwa` or `v4l2` Rust crate to pull single frames from each camera's RTSP URL on motion trigger. Three cameras = three `Vision` trait impls, or one impl that round-robins based on which camera detected motion. The architectural call lands in the v0.2 brainstorming session.

## Other hardware — Phase 0 procurement, still open

- **Cash recycler.** Glory CI-10 default (4–8 week lead time, order day-one of v0.2.0 procurement). Alternatives: CI-5 (smaller footprint), SUZOHAPP Eagle, Volumatic CounterCache.
- **Boundary microphone.** TBD. Phase 1 ships push-to-talk on the cashier tablet; ambient mic + VAD comes once the shop's noise profile is characterized.
- **Customer display.** 15–22" monitor on a swivel stand. Capacitive touch optional for Phase 1.
- **Cashier tablet.** Cheap Android tablet on a stand near the till. Same PWA as the customer display, different route.
- **Compute.** Strix Halo (Ryzen AI MAX+ 395) workstation — already on hand.
