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

## license

mit
