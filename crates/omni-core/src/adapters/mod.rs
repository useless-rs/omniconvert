pub mod external;
pub mod native_archive;
pub mod native_image;
pub mod native_text;

use std::path::Path;

use crate::error::Result;

/// Plugin surface: every format pair is an Adapter.
/// New pairs ship as a new impl WITHOUT touching core (see README).
#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn supports(&self, from: &str, to: &str) -> bool;
    async fn convert(&self, from: &str, to: &str, input: &Path, output: &Path) -> Result<()>;
}
