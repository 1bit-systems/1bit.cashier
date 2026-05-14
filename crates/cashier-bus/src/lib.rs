// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

//! Event bus types and trait for the 1bit.cashier subsystems.

pub mod language;

#[cfg(test)]
mod language_tests {
    use crate::language::Language;

    #[test]
    fn serializes_english_as_en() {
        let json = serde_json::to_string(&Language::English).unwrap();
        assert_eq!(json, "\"en\"");
    }

    #[test]
    fn serializes_mikmaq_listuguj_as_mikq_lj() {
        let json = serde_json::to_string(&Language::MikmaqListuguj).unwrap();
        assert_eq!(json, "\"mikq-LJ\"");
    }

    #[test]
    fn default_is_english() {
        assert_eq!(Language::default(), Language::English);
    }
}

pub mod events;

#[cfg(test)]
mod event_tests {
    use crate::events::{Event, EventEnvelope, VisionDetection};
    use crate::language::Language;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn envelope_round_trips_through_json() {
        let envelope = EventEnvelope {
            ts: Utc::now(),
            transaction_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            language: Language::English,
            payload: Event::VisionDetection(VisionDetection {
                sku: Some("DEMO-001".to_string()),
                confidence: 0.92,
                candidates: vec![],
                frame_uri: "audit://demo".to_string(),
            }),
        };

        let json = serde_json::to_string(&envelope).unwrap();
        let back: EventEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(envelope.transaction_id, back.transaction_id);
        assert_eq!(envelope.language, back.language);
        match back.payload {
            Event::VisionDetection(v) => {
                assert_eq!(v.sku.as_deref(), Some("DEMO-001"));
                assert!((v.confidence - 0.92).abs() < 1e-6);
            }
            _ => panic!("expected VisionDetection"),
        }
    }
}
