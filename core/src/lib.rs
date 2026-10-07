//! Odomouse core: everything that decides *what* to count and how to
//! store and summarise it. The native shells (macOS, Windows, Linux) only
//! feed input events in and show the JSON that comes out.

pub mod breaks;
pub mod compare;
pub mod day;
pub mod displays;
pub mod engine;
pub mod ffi;
pub mod fun;
pub mod i18n;
pub mod json;
pub mod keyboard;
pub mod shortcuts;
pub mod store;
pub mod time;
pub mod tracker;
pub mod units;
