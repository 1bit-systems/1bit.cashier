// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use cashier_bus::events::{Event, EventEnvelope, IntentAction, AudioIntent, VisionDetection};
use cashier_bus::language::Language;
use cashier_bus::memory::MemoryBus;
use cashier_bus::EventBus;
use cashier_order::engine::{Catalog, CatalogEntry, OrderEngine};
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

fn demo_catalog() -> Catalog {
    Catalog::from_entries(vec![
        CatalogEntry { sku: "DEMO-001".into(), name: "Coffee".into(), unit_price_cents: 350 },
        CatalogEntry { sku: "DEMO-002".into(), name: "Croissant".into(), unit_price_cents: 425 },
    ])
}

fn vision_envelope(sku: &str, tx_id: Uuid) -> EventEnvelope {
    EventEnvelope {
        ts: Utc::now(),
        transaction_id: tx_id,
        correlation_id: Uuid::new_v4(),
        language: Language::English,
        payload: Event::VisionDetection(VisionDetection {
            sku: Some(sku.into()),
            confidence: 0.99,
            candidates: vec![],
            frame_uri: "audit://t".into(),
        }),
    }
}

#[tokio::test]
async fn vision_detection_adds_to_cart_and_emits_order_update() {
    let bus = Arc::new(MemoryBus::new(32));
    let catalog = demo_catalog();
    let mut updates = bus.subscribe();
    let _engine = OrderEngine::spawn(bus.clone(), catalog);

    let tx_id = Uuid::new_v4();
    bus.publish(vision_envelope("DEMO-001", tx_id)).await.unwrap();

    let env = timeout(Duration::from_millis(500), async {
        loop {
            let e = updates.recv().await.unwrap();
            if matches!(e.payload, Event::OrderUpdate(_)) {
                return e;
            }
        }
    })
    .await
    .expect("OrderUpdate not received within 500ms");

    match env.payload {
        Event::OrderUpdate(u) => {
            assert_eq!(u.state, "building");
            assert_eq!(u.lines.len(), 1);
            assert_eq!(u.lines[0].sku, "DEMO-001");
            assert_eq!(u.total_cents, 350);
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn audio_change_qty_intent_updates_cart() {
    let bus = Arc::new(MemoryBus::new(32));
    let catalog = demo_catalog();
    let mut updates = bus.subscribe();
    let _engine = OrderEngine::spawn(bus.clone(), catalog);

    let tx_id = Uuid::new_v4();
    bus.publish(vision_envelope("DEMO-001", tx_id)).await.unwrap();

    // Wait for the OrderUpdate the vision event produced
    let _ = timeout(Duration::from_millis(500), async {
        loop {
            let e = updates.recv().await.unwrap();
            if matches!(e.payload, Event::OrderUpdate(_)) {
                return e;
            }
        }
    })
    .await
    .unwrap();

    let intent = EventEnvelope {
        ts: Utc::now(),
        transaction_id: tx_id,
        correlation_id: Uuid::new_v4(),
        language: Language::English,
        payload: Event::AudioIntent(AudioIntent {
            transcript: "make it three".into(),
            actions: vec![IntentAction::ChangeQty { sku: "DEMO-001".into(), qty: 3 }],
        }),
    };
    bus.publish(intent).await.unwrap();

    let env = timeout(Duration::from_millis(500), async {
        loop {
            let e = updates.recv().await.unwrap();
            if let Event::OrderUpdate(ref u) = e.payload {
                if u.total_cents == 350 * 3 {
                    return e;
                }
            }
        }
    })
    .await
    .expect("updated OrderUpdate not received");

    match env.payload {
        Event::OrderUpdate(u) => {
            assert_eq!(u.lines.len(), 1);
            assert_eq!(u.lines[0].sku, "DEMO-001");
            assert_eq!(u.lines[0].qty, 3);
            assert_eq!(u.total_cents, 1050);
        }
        _ => unreachable!(),
    }
}
