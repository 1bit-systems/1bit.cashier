// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use cashier_bus::events::{Event, EventEnvelope, VisionDetection};
use cashier_bus::language::Language;
use cashier_bus::memory::MemoryBus;
use cashier_bus::EventBus;
use chrono::Utc;
use uuid::Uuid;

fn sample_envelope() -> EventEnvelope {
    EventEnvelope {
        ts: Utc::now(),
        transaction_id: Uuid::new_v4(),
        correlation_id: Uuid::new_v4(),
        language: Language::English,
        payload: Event::VisionDetection(VisionDetection {
            sku: Some("SKU-1".into()),
            confidence: 0.99,
            candidates: vec![],
            frame_uri: "audit://test".into(),
        }),
    }
}

#[tokio::test]
async fn published_envelope_is_received_by_subscriber() {
    let bus = MemoryBus::new(16);
    let mut rx = bus.subscribe();

    let env = sample_envelope();
    let tx_id = env.transaction_id;

    bus.publish(env).await.unwrap();
    let received = rx.recv().await.unwrap();

    assert_eq!(received.transaction_id, tx_id);
}

#[tokio::test]
async fn multiple_subscribers_each_receive_the_envelope() {
    let bus = MemoryBus::new(16);
    let mut rx_a = bus.subscribe();
    let mut rx_b = bus.subscribe();

    bus.publish(sample_envelope()).await.unwrap();

    let a = rx_a.recv().await.unwrap();
    let b = rx_b.recv().await.unwrap();
    assert_eq!(a.transaction_id, b.transaction_id);
}
