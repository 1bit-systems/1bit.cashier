// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::language::Language;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub ts: DateTime<Utc>,
    pub transaction_id: Uuid,
    pub correlation_id: Uuid,
    pub language: Language,
    pub payload: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    VisionDetection(VisionDetection),
    AudioIntent(AudioIntent),
    OrderUpdate(OrderUpdate),
    CashEvent(CashEvent),
    AuditWrite(AuditWrite),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionDetection {
    pub sku: Option<String>,
    pub confidence: f32,
    pub candidates: Vec<SkuCandidate>,
    pub frame_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkuCandidate {
    pub sku: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioIntent {
    pub transcript: String,
    pub actions: Vec<IntentAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum IntentAction {
    Add { sku: String, qty: u32 },
    Remove { sku: String },
    ChangeQty { sku: String, qty: u32 },
    Void,
    Clarify { question: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub state: String,
    pub lines: Vec<OrderLine>,
    pub total_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLine {
    pub sku: String,
    pub name: String,
    pub qty: u32,
    pub unit_price_cents: u64,
    pub line_total_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cash", rename_all = "snake_case")]
pub enum CashEvent {
    Charged { amount_cents: u64, tendered_cents: u64, change_cents: u64 },
    Jam { description: String },
    DoorOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditWrite {
    pub record: String,
    pub body: serde_json::Value,
}
