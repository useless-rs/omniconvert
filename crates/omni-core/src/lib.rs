pub mod adapters;
pub mod deps;
pub mod detect;
pub mod engine;
pub mod error;
pub mod formats;
pub mod presets;
pub mod queue;
pub mod registry;

pub use detect::{detect, Detected};
pub use engine::{convert_file, OmniEngine};
pub use error::{ConverterError, Result};
pub use formats::{all_formats, find_by_extension, Format};
pub use presets::{all_presets, builtin_presets, Preset};
pub use queue::{Job, JobQueue, JobState};
pub use registry::{ConversionGraph, Edge};
