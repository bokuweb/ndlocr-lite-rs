pub mod app;
pub use app::{PageOptions, PageResult, recognize_page_gray_u8, recognize_page_rgb_u8};
mod app_trace;
pub mod cascade;
pub mod cli;
pub mod infer;
pub mod io;
pub mod output;
pub mod pipeline;
pub mod postprocess;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod web;
