pub mod cli;
pub mod instances;

use rust_embed::RustEmbed;

/// Embeds the contents of the `files/` directory into the binary
///
/// This struct contains the static data used within `eksnode`
#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;
