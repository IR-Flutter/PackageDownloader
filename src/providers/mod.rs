//! Ecosystem-specific providers.

pub mod android;
pub mod dart;
pub mod flutter;
pub mod gradle;
pub mod ios;

pub use android::AndroidProvider;
pub use dart::DartProvider;
pub use flutter::FlutterProvider;
pub use gradle::GradleProvider;
pub use ios::IosProvider;

use crate::core::provider::Provider;

pub fn all_providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(FlutterProvider),
        Box::new(DartProvider),
        Box::new(GradleProvider),
        Box::new(AndroidProvider),
        Box::new(IosProvider),
    ]
}

pub fn get_provider(id: &str) -> Option<Box<dyn Provider>> {
    match id.to_lowercase().as_str() {
        "flutter" => Some(Box::new(FlutterProvider)),
        "dart" | "pub" => Some(Box::new(DartProvider)),
        "gradle" => Some(Box::new(GradleProvider)),
        "android" => Some(Box::new(AndroidProvider)),
        "ios" => Some(Box::new(IosProvider)),
        _ => None,
    }
}
