// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::error::OrderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartLine {
    pub sku: String,
    pub name: String,
    pub qty: u32,
    pub unit_price_cents: u64,
}

impl CartLine {
    pub fn line_total_cents(&self) -> u64 {
        self.unit_price_cents.saturating_mul(self.qty as u64)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Cart {
    lines: Vec<CartLine>,
}

impl Cart {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn lines(&self) -> &[CartLine] {
        &self.lines
    }

    pub fn total_cents(&self) -> u64 {
        self.lines.iter().map(CartLine::line_total_cents).sum()
    }

    pub fn add_item(&mut self, sku: &str, name: &str, qty: u32, unit_price_cents: u64) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.sku == sku) {
            line.qty = line.qty.saturating_add(qty);
        } else {
            self.lines.push(CartLine {
                sku: sku.to_string(),
                name: name.to_string(),
                qty,
                unit_price_cents,
            });
        }
    }

    pub fn change_qty(&mut self, sku: &str, qty: u32) -> Result<(), OrderError> {
        let idx = self
            .lines
            .iter()
            .position(|l| l.sku == sku)
            .ok_or_else(|| OrderError::SkuNotFound(sku.to_string()))?;
        if qty == 0 {
            self.lines.remove(idx);
        } else {
            self.lines[idx].qty = qty;
        }
        Ok(())
    }

    pub fn remove_item(&mut self, sku: &str) -> Result<CartLine, OrderError> {
        let idx = self
            .lines
            .iter()
            .position(|l| l.sku == sku)
            .ok_or_else(|| OrderError::SkuNotFound(sku.to_string()))?;
        Ok(self.lines.remove(idx))
    }
}
