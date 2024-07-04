pub mod cli;
pub mod inputs;

pub use cli::Cli;
use rust_embed::RustEmbed;

/// Embeds the contents of the `files/` directory into the binary
///
/// This struct contains the static data used within `eksnode`
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;
