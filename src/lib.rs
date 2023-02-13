#![cfg_attr(docsrs, doc = include_str!("../docs/index.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use include_sql::include_sql;

#[doc(hidden)]
pub mod util;

#[cfg(feature = "tokio")]
#[doc(hidden)]
pub mod async_await;

#[cfg(not(feature = "tokio"))]
#[doc(hidden)]
pub mod sync;
