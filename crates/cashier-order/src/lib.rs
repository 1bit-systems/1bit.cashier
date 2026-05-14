// 1bit.cashier — Cash-only autonomous POS
// Copyright (C) 2026 1bit.cashier contributors
// SPDX-License-Identifier: AGPL-3.0-only

//! Cart and order state machine.

pub mod cart;
pub mod error;

pub use cart::{Cart, CartLine};
pub use error::OrderError;

#[cfg(test)]
mod cart_tests {
    use super::*;

    #[test]
    fn empty_cart_has_zero_total() {
        let cart = Cart::new();
        assert!(cart.is_empty());
        assert_eq!(cart.total_cents(), 0);
        assert!(cart.lines().is_empty());
    }

    #[test]
    fn add_item_appends_a_new_line() {
        let mut cart = Cart::new();
        cart.add_item("DEMO-001", "Coffee, medium", 1, 350);
        assert_eq!(cart.lines().len(), 1);
        assert_eq!(cart.lines()[0].sku, "DEMO-001");
        assert_eq!(cart.lines()[0].qty, 1);
        assert_eq!(cart.total_cents(), 350);
    }

    #[test]
    fn add_item_increments_existing_line() {
        let mut cart = Cart::new();
        cart.add_item("DEMO-001", "Coffee", 1, 350);
        cart.add_item("DEMO-001", "Coffee", 2, 350);
        assert_eq!(cart.lines().len(), 1);
        assert_eq!(cart.lines()[0].qty, 3);
        assert_eq!(cart.total_cents(), 350 * 3);
    }

    #[test]
    fn change_qty_updates_existing_line() {
        let mut cart = Cart::new();
        cart.add_item("DEMO-001", "Coffee", 1, 350);
        cart.change_qty("DEMO-001", 5).unwrap();
        assert_eq!(cart.lines()[0].qty, 5);
    }

    #[test]
    fn change_qty_on_missing_sku_errors() {
        let mut cart = Cart::new();
        let err = cart.change_qty("MISSING", 1).unwrap_err();
        assert_eq!(err, OrderError::SkuNotFound("MISSING".to_string()));
    }

    #[test]
    fn change_qty_to_zero_removes_the_line() {
        let mut cart = Cart::new();
        cart.add_item("DEMO-001", "Coffee", 1, 350);
        cart.change_qty("DEMO-001", 0).unwrap();
        assert!(cart.is_empty());
    }

    #[test]
    fn remove_item_drops_a_line() {
        let mut cart = Cart::new();
        cart.add_item("DEMO-001", "Coffee", 2, 350);
        cart.add_item("DEMO-002", "Croissant", 1, 425);
        let removed = cart.remove_item("DEMO-001").unwrap();
        assert_eq!(removed.sku, "DEMO-001");
        assert_eq!(cart.lines().len(), 1);
        assert_eq!(cart.lines()[0].sku, "DEMO-002");
    }
}
