#[cfg(feature = "icon")]
mod icon;

#[cfg(feature = "markdown")]
mod markdown;

#[cfg(feature = "icon")]
pub use icon::icon;

#[cfg(feature = "markdown")]
pub use markdown::markdown;
