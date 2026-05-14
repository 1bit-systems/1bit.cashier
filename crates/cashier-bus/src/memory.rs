// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::bus::EventBus;
use crate::error::BusError;
use crate::events::EventEnvelope;
use async_trait::async_trait;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone)]
pub struct MemoryBus {
    tx: Sender<EventEnvelope>,
}

impl MemoryBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }
}

#[async_trait]
impl EventBus for MemoryBus {
    async fn publish(&self, env: EventEnvelope) -> Result<(), BusError> {
        match self.tx.send(env) {
            Ok(_) => Ok(()),
            // No active subscribers is not an error for v0.1 — the bench
            // path may publish before any consumer attaches. Document this
            // in EventBus::publish's docstring later if it becomes load-bearing.
            Err(_) => Ok(()),
        }
    }

    fn subscribe(&self) -> Receiver<EventEnvelope> {
        self.tx.subscribe()
    }
}
