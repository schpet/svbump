# svbump

a simple cli tool for bumping semantic versions in various config file formats.

## supported formats

- json
- toml
- yaml

## usage

```bash
# bump the patch version in package.json
svbump patch version package.json

# bump the minor version in a nested field
svbump minor package.version Cargo.toml

# bump the major version in a yaml file
svbump major version app.yaml

# set a specific version (must be higher than current)
svbump 2.5.0 version package.json

# read the current version without modifying
svbump read version package.json
```

## installation

todo

## development

run tests:
```bash
cargo test
```

build locally:
```bash
cargo build
```
