// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BusError {
    #[error("no active subscribers")]
    NoSubscribers,
    #[error("channel closed")]
    Closed,
}
