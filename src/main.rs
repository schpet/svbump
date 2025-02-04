use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use semver::Version;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Value as TomlValue};

#[derive(Debug, Clone, ValueEnum)]
enum VersionBump {
    Major,
    Minor,
    Patch,
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Version segment to update (major, minor, patch)
    #[arg(value_enum)]
    level: Option<VersionBump>,

    /// Field selector using dot notation (e.g. "package.version")
    selector: Option<String>,

    /// Path to the file to process
    file: Option<PathBuf>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Read current version
    Read {
        /// Field selector using dot notation (e.g. "package.version")
        selector: String,

        /// Path to the file to process
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Some(Command::Read { selector, file }) => {
            let path = file.as_path();
            let content = fs::read_to_string(path)?;
            
            let version = match get_file_type(path) {
                "toml" => {
                    let doc = content.parse::<DocumentMut>()?;
                    read_version_toml(&doc, &selector)?
                }
                "yml" | "yaml" => {
                    let value: YamlValue = serde_yaml::from_str(&content)?;
                    read_version_yaml(&value, &selector)?
                }
                _ => {
                    let value: JsonValue = serde_json::from_str(&content)?;
                    read_version_json(&value, &selector)?
                }
            };
            println!("{}", version);
        }
        None => {
            // Default to bump command
            let level = args.level.ok_or_else(|| anyhow::anyhow!("Version bump level is required"))?;
            let selector = args.selector.ok_or_else(|| anyhow::anyhow!("Selector is required"))?;
            let file = args.file.ok_or_else(|| anyhow::anyhow!("File path is required"))?;
            let path = file.as_path();
            let content = fs::read_to_string(path)?;

            match get_file_type(path) {
                "toml" => {
                    let mut doc = content.parse::<DocumentMut>()?;
                    bump_version_toml(&mut doc, &selector, &level)?;
                    fs::write(path, doc.to_string())?;
                }
                "yml" | "yaml" => {
                    let mut value: YamlValue = serde_yaml::from_str(&content)?;
                    bump_version_yaml(&mut value, &selector, &level)?;
                    fs::write(path, serde_yaml::to_string(&value)?)?;
                }
                _ => {
                    let mut value: JsonValue = serde_json::from_str(&content)?;
                    bump_version_json(&mut value, &selector, &level)?;
                    fs::write(path, serde_json::to_string_pretty(&value)?)?;
                }
            }
        }
    }

    Ok(())
}

fn get_file_type(path: &Path) -> &str {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("json")
}

fn bump_semver(version: &str, level: &VersionBump) -> Result<String> {
    let mut v = Version::parse(version)?;

    match level {
        VersionBump::Major => {
            v.major += 1;
            v.minor = 0;
            v.patch = 0;
        }
        VersionBump::Minor => {
            v.minor += 1;
            v.patch = 0;
        }
        VersionBump::Patch => v.patch += 1,
    }

    // Preserve any existing pre-release and build metadata
    Ok(format!("{}.{}.{}", v.major, v.minor, v.patch))
}

fn bump_version_toml(
    doc: &mut DocumentMut,
    selector: &str,
    level: &VersionBump,
) -> Result<()> {
    let path_parts: Vec<&str> = selector.split('.').collect();
    let mut current = doc.as_table_mut();

    for part in &path_parts[..path_parts.len() - 1] {
        current = current[part]
            .as_table_mut()
            .with_context(|| format!("No table found at selector {}", part))?;
    }

    let last_part = path_parts.last().unwrap();
    let version = current[last_part]
        .as_str()
        .with_context(|| format!("No string value found at {}", selector))?;

    let new_version = bump_semver(version, level)?;
    current[last_part] = Item::Value(TomlValue::from(new_version));
    Ok(())
}

fn walk_json_mut<'a>(value: &'a mut JsonValue, parts: &[&str]) -> Result<&'a mut JsonValue> {
    let part = parts[0];
    let value = value.get_mut(part)
        .with_context(|| format!("Missing key: {}", part))?;

    if parts.len() == 1 {
        Ok(value)
    } else {
        walk_json_mut(value, &parts[1..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_version_bump() -> Result<()> {
        let json_content = r#"{
            "name": "test-package",
            "version": "1.2.3"
        }"#;
        
        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, json_content)?;

        let args = Args {
            command: None,
            level: Some(VersionBump::Patch),
            selector: Some("version".to_string()),
            file: Some(temp_file.path().to_path_buf()),
        };

        let content = fs::read_to_string(temp_file.path())?;
        let mut value: JsonValue = serde_json::from_str(&content)?;
        bump_version_json(&mut value, args.selector.as_ref().unwrap(), args.level.as_ref().unwrap())?;
        
        assert_eq!(value["version"], "1.2.4");
        Ok(())
    }

    #[test]
    fn test_toml_version_bump() -> Result<()> {
        let toml_content = r#"
[package]
name = "test-package"
version = "1.2.3"
"#;
        
        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, toml_content)?;

        let args = Args {
            command: None,
            level: Some(VersionBump::Minor),
            selector: Some("package.version".to_string()),
            file: Some(temp_file.path().to_path_buf()),
        };

        let content = fs::read_to_string(temp_file.path())?;
        let mut doc = content.parse::<DocumentMut>()?;
        bump_version_toml(&mut doc, args.selector.as_ref().unwrap(), args.level.as_ref().unwrap())?;
        
        assert_eq!(doc["package"]["version"].as_str().unwrap(), "1.3.0");
        Ok(())
    }

    #[test]
    fn test_yaml_version_bump() -> Result<()> {
        let yaml_content = r#"
name: test-package
version: 1.2.3
"#;
        
        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, yaml_content)?;

        let args = Args {
            command: None,
            level: Some(VersionBump::Major),
            selector: Some("version".to_string()),
            file: Some(temp_file.path().to_path_buf()),
        };

        let content = fs::read_to_string(temp_file.path())?;
        let mut value: YamlValue = serde_yaml::from_str(&content)?;
        bump_version_yaml(&mut value, args.selector.as_ref().unwrap(), args.level.as_ref().unwrap())?;
        
        assert_eq!(value["version"].as_str().unwrap(), "2.0.0");
        Ok(())
    }
}

fn walk_yaml_mut<'a>(value: &'a mut YamlValue, parts: &[&str]) -> Result<&'a mut YamlValue> {
    let part = parts[0];
    let value = value.get_mut(part)
        .with_context(|| format!("Missing key: {}", part))?;

    if parts.len() == 1 {
        Ok(value)
    } else {
        walk_yaml_mut(value, &parts[1..])
    }
}

fn bump_version_yaml(
    value: &mut YamlValue,
    selector: &str,
    bump: &VersionBump,
) -> Result<()> {
    let parts: Vec<&str> = selector.split('.').collect();
    let target = walk_yaml_mut(value, &parts)?;

    let version = target.as_str()
        .with_context(|| format!("Version field is not a string at {}", selector))?;

    let new_version = bump_semver(version, bump)?;
    *target = YamlValue::String(new_version);
    Ok(())
}

fn bump_version_json(
    value: &mut JsonValue,
    selector: &str,
    bump: &VersionBump,
) -> Result<()> {
    let parts: Vec<&str> = selector.split('.').collect();
    let target = walk_json_mut(value, &parts)?;

    let version = target.as_str()
        .with_context(|| format!("Version field is not a string at {}", selector))?;

    let new_version = bump_semver(version, bump)?;
    *target = JsonValue::String(new_version);
    Ok(())
}
fn walk_json<'a>(value: &'a JsonValue, parts: &[&str]) -> Result<&'a JsonValue> {
    let part = parts[0];
    let value = value.get(part)
        .with_context(|| format!("Missing key: {}", part))?;

    if parts.len() == 1 {
        Ok(value)
    } else {
        walk_json(value, &parts[1..])
    }
}

fn walk_yaml<'a>(value: &'a YamlValue, parts: &[&str]) -> Result<&'a YamlValue> {
    let part = parts[0];
    let value = value.get(part)
        .with_context(|| format!("Missing key: {}", part))?;

    if parts.len() == 1 {
        Ok(value)
    } else {
        walk_yaml(value, &parts[1..])
    }
}

fn read_version_json(value: &JsonValue, selector: &str) -> Result<String> {
    let parts: Vec<&str> = selector.split('.').collect();
    let target = walk_json(value, &parts)?;

    target.as_str()
        .with_context(|| format!("Version field is not a string at {}", selector))
        .map(String::from)
}

fn read_version_yaml(value: &YamlValue, selector: &str) -> Result<String> {
    let parts: Vec<&str> = selector.split('.').collect();
    let target = walk_yaml(value, &parts)?;

    target.as_str()
        .with_context(|| format!("Version field is not a string at {}", selector))
        .map(String::from)
}

fn read_version_toml(doc: &DocumentMut, selector: &str) -> Result<String> {
    let path_parts: Vec<&str> = selector.split('.').collect();
    let mut current = doc.as_table();

    for part in &path_parts[..path_parts.len() - 1] {
        current = current.get(*part)
            .and_then(|v| v.as_table())
            .with_context(|| format!("No table found at selector {}", part))?;
    }

    let last_part = path_parts.last().unwrap();
    current.get(*last_part)
        .and_then(|v| v.as_str())
        .with_context(|| format!("No string value found at {}", selector))
        .map(String::from)
}
