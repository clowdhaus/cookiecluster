pub mod cli;
pub mod inputs;
pub mod instances;

pub use cli::Cli;
pub use inputs::Inputs;
pub use instances::INSTANCE_TYPES;
use rust_embed::RustEmbed;

/// Embeds the contents of the `files/` directory into the binary
///
/// This struct contains the static data used within `eksnode`
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;
