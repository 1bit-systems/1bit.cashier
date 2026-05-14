// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

use crate::error::OrderError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrderState {
    #[default]
    Idle,
    Building,
    Review,
    AwaitingPayment,
    Paying,
    Complete,
}

impl OrderState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Building => "building",
            Self::Review => "review",
            Self::AwaitingPayment => "awaiting-payment",
            Self::Paying => "paying",
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    StartBuilding,
    StartReview,
    StartAwaitingPayment,
    StartPaying,
    Finalize,
    Reset,
}

#[derive(Debug, Default)]
pub struct StateMachine {
    state: OrderState,
}

impl StateMachine {
    pub fn new() -> Self {
        Self { state: OrderState::Idle }
    }

    pub fn state(&self) -> OrderState {
        self.state
    }

    pub fn apply(&mut self, t: Transition) -> Result<(), OrderError> {
        use OrderState::*;
        use Transition::*;

        let next = match (self.state, t) {
            (Idle, StartBuilding) => Building,
            (Building, StartReview) => Review,
            (Review, StartAwaitingPayment) => AwaitingPayment,
            (AwaitingPayment, StartPaying) => Paying,
            (Paying, Finalize) => Complete,
            (Complete, Reset) => Idle,
            (Idle, Reset) => Idle,
            (from, t) => {
                return Err(OrderError::InvalidTransition {
                    from: from.as_str().to_string(),
                    to: format!("{:?}", t),
                });
            }
        };

        self.state = next;
        Ok(())
    }
}
