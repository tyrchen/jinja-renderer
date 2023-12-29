#[cfg(any(feature = "with-axum", feature = "with-axum-06"))]
mod axum;

#[cfg(all(feature = "with-axum", feature = "with-axum-06"))]
compile_error!("feature \"foo\" and feature \"bar\" cannot be enabled at the same time");
