// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::cart::Cart;
use crate::state::{OrderState, StateMachine, Transition};
use cashier_bus::events::{Event, EventEnvelope, IntentAction, OrderLine, OrderUpdate};
use cashier_bus::EventBus;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub sku: String,
    pub name: String,
    pub unit_price_cents: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    by_sku: HashMap<String, CatalogEntry>,
}

impl Catalog {
    pub fn from_entries(entries: Vec<CatalogEntry>) -> Self {
        let mut by_sku = HashMap::new();
        for entry in entries {
            by_sku.insert(entry.sku.clone(), entry);
        }
        Self { by_sku }
    }

    pub fn lookup(&self, sku: &str) -> Option<&CatalogEntry> {
        self.by_sku.get(sku)
    }
}

pub struct OrderEngine {
    handle: JoinHandle<()>,
}

impl OrderEngine {
    pub fn spawn<B>(bus: Arc<B>, catalog: Catalog) -> Self
    where
        B: EventBus + 'static,
    {
        let mut rx = bus.subscribe();
        let bus_for_task = bus.clone();
        let handle = tokio::spawn(async move {
            let mut cart = Cart::new();
            let mut machine = StateMachine::new();
            let mut current_tx: Option<Uuid> = None;

            while let Ok(env) = rx.recv().await {
                let dirty = match &env.payload {
                    Event::VisionDetection(v) => {
                        if let Some(sku) = &v.sku {
                            if let Some(entry) = catalog.lookup(sku) {
                                if machine.state() == OrderState::Idle {
                                    let _ = machine.apply(Transition::StartBuilding);
                                }
                                current_tx.get_or_insert(env.transaction_id);
                                cart.add_item(&entry.sku, &entry.name, 1, entry.unit_price_cents);
                                true
                            } else {
                                tracing::warn!("vision: unknown sku {}", sku);
                                false
                            }
                        } else {
                            false
                        }
                    }
                    Event::AudioIntent(intent) => {
                        let mut changed = false;
                        for action in &intent.actions {
                            match action {
                                IntentAction::Add { sku, qty } => {
                                    if let Some(entry) = catalog.lookup(sku) {
                                        cart.add_item(
                                            &entry.sku,
                                            &entry.name,
                                            *qty,
                                            entry.unit_price_cents,
                                        );
                                        changed = true;
                                    }
                                }
                                IntentAction::Remove { sku } => {
                                    if cart.remove_item(sku).is_ok() {
                                        changed = true;
                                    }
                                }
                                IntentAction::ChangeQty { sku, qty } => {
                                    if cart.change_qty(sku, *qty).is_ok() {
                                        changed = true;
                                    }
                                }
                                IntentAction::Void => {
                                    cart = Cart::new();
                                    let _ = machine.apply(Transition::Reset);
                                    current_tx = None;
                                    changed = true;
                                }
                                IntentAction::Clarify { question } => {
                                    tracing::info!("audio clarify: {}", question);
                                }
                            }
                        }
                        changed
                    }
                    _ => false,
                };

                if dirty {
                    let lines = cart
                        .lines()
                        .iter()
                        .map(|l| OrderLine {
                            sku: l.sku.clone(),
                            name: l.name.clone(),
                            qty: l.qty,
                            unit_price_cents: l.unit_price_cents,
                            line_total_cents: l.line_total_cents(),
                        })
                        .collect();
                    let update = OrderUpdate {
                        state: machine.state().as_str().to_string(),
                        lines,
                        total_cents: cart.total_cents(),
                    };
                    let envelope = EventEnvelope {
                        ts: Utc::now(),
                        transaction_id: current_tx.unwrap_or_else(Uuid::new_v4),
                        correlation_id: env.correlation_id,
                        language: env.language,
                        payload: Event::OrderUpdate(update),
                    };
                    let _ = bus_for_task.publish(envelope).await;
                }
            }
        });
        Self { handle }
    }

    pub fn join_handle(&self) -> &JoinHandle<()> {
        &self.handle
    }
}
