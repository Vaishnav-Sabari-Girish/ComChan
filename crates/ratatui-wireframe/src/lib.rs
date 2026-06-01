#![no_std]
extern crate alloc;

pub mod model;
pub mod widget;

// Re-export the widget so other crates can access it directly at the root level
pub use widget::WireframeWidget;
