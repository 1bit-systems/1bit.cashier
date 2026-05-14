// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "mikq-LJ")]
    MikmaqListuguj,
}
