// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::error::BusError;
use crate::events::EventEnvelope;
use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, env: EventEnvelope) -> Result<(), BusError>;
    fn subscribe(&self) -> Receiver<EventEnvelope>;
}
