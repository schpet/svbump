# svbump

a simple cli tool for bumping semantic versions in various config file formats.

## supported formats

- json
- toml
- yaml

## usage

```sh
svbump write [LEVEL] [SELECTOR] [FILE]   # modify version
svbump read [SELECTOR] [FILE]            # read version
svbump preview [LEVEL] [SELECTOR] [FILE] # preview change
```

### examples

```sh
# bump the patch version in package.json
svbump write patch version package.json

# bump the minor version in a nested field
svbump write minor package.version Cargo.toml

# bump the major version in a yaml file
svbump write major version app.yaml

# set a specific version (must be higher than current)
svbump write 2.5.0 version package.json

# preview what a bump would do without modifying
svbump preview minor version package.json

# print the current version to stdout
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
