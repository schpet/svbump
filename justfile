default:
    @just -l -u

# release a major, minor or patch version
tag:
    svbump write "$(changelog version latest)" package.version Cargo.toml
    cargo check
    git commit CHANGELOG.md Cargo.toml Cargo.lock -m "chore: Release svbump version $(svbump read package.version Cargo.toml)"
    git tag "v$(svbump read package.version Cargo.toml)"

    @echo "tagged v$(svbump read package.version Cargo.toml)"
    @echo
    @echo "run this to release it:"
    @echo
    @echo "  git push origin HEAD --tags"
