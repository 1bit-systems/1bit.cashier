// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrderError {
    #[error("sku not in cart: {0}")]
    SkuNotFound(String),
    #[error("invalid quantity: {0}")]
    InvalidQty(u32),
    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
}
