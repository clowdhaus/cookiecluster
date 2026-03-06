/// Validate that a name contains only safe characters for use in
/// Terraform resource names and filesystem paths.
///
/// Returns `Result<(), String>` for use with both `dialoguer` validators
/// and `anyhow::ensure!` (via `.map_err()`).
pub fn name(input: &str) -> Result<(), String> {
  if input.is_empty() {
    return Err("Name cannot be empty".to_string());
  }
  if input.contains("..") {
    return Err("Name must not contain '..'".to_string());
  }
  if !input
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
  {
    return Err("Name must contain only alphanumeric characters, hyphens, underscores, and dots".to_string());
  }
  Ok(())
}

/// Validate that a filter pattern contains only safe characters
pub fn filter(input: &str) -> Result<(), String> {
  if input.is_empty() {
    return Err("Filter cannot be empty".to_string());
  }
  if !input
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '*' || c == '.')
  {
    return Err(
      "Filter must contain only alphanumeric characters, hyphens, underscores, dots, and wildcards (*)".to_string(),
    );
  }
  Ok(())
}

/// Validate that an availability zone looks valid
pub fn availability_zone(input: &str) -> Result<(), String> {
  if input.is_empty() {
    return Err("Availability zone cannot be empty".to_string());
  }
  if !input.chars().all(|c| c.is_alphanumeric() || c == '-') {
    return Err("Availability zone must contain only alphanumeric characters and hyphens".to_string());
  }
  Ok(())
}

/// Validate that an instance type contains only safe characters
pub fn instance_type(input: &str) -> Result<(), String> {
  if input.is_empty() {
    return Err("Instance type cannot be empty".to_string());
  }
  if !input.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
    return Err("Instance type must contain only alphanumeric characters, dots, and hyphens".to_string());
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_name() {
    assert!(name("valid-name_123").is_ok());
    assert!(name("my-cluster").is_ok());
    assert!(name("prod.us-west-2").is_ok());
    assert!(name("").is_err());
    assert!(name("bad name").is_err());
    assert!(name("../../etc").is_err());
    assert!(name("${file(\"/etc/shadow\")}").is_err());
  }

  #[test]
  fn test_filter() {
    assert!(filter("*-private-*").is_ok());
    assert!(filter("my-vpc-data-*").is_ok());
    assert!(filter("").is_err());
    assert!(filter("${bad}").is_err());
    assert!(filter("has spaces").is_err());
  }

  #[test]
  fn test_availability_zone() {
    assert!(availability_zone("us-west-2a").is_ok());
    assert!(availability_zone("eu-central-1b").is_ok());
    assert!(availability_zone("").is_err());
    assert!(availability_zone("us west 2a").is_err());
    assert!(availability_zone("${data.az}").is_err());
  }

  #[test]
  fn test_instance_type() {
    assert!(instance_type("m7a.xlarge").is_ok());
    assert!(instance_type("p5.48xlarge").is_ok());
    assert!(instance_type("").is_err());
    assert!(instance_type("${file(\"/etc/shadow\")}").is_err());
  }
}
