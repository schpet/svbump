# svbump

a simple cli tool for bumping semantic versions in various config file formats.

## supported formats

- json
- toml
- yaml

## usage

```sh
svbump [LEVEL] [SELECTOR] [FILE] # bumping
svbump read [SELECTOR] [FILE]    # reading
```
### examples

```sh
# bump the patch version in package.json
svbump patch version package.json

# bump the minor version in a nested field
svbump minor package.version Cargo.toml

# bump the major version in a yaml file
svbump major version app.yaml

# set a specific version (must be higher than current)
svbump 2.5.0 version package.json

# print the current version to stdout without modifying
svbump read version package.json
svbump read package.version Cargo.toml
```

## installation

### homebrew

```sh
brew install schpet/tap/svbump
```

### binaries

head on over to https://github.com/schpet/svbump/releases/latest

## similar tools

### semver-bump

https://github.com/ceejbot/semver-bump

bumps a semver from stdin, can be composed with other stuff like [tomato for toml](https://github.com/ceejbot/tomato) to update files.
