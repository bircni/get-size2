#!/usr/bin/env bash

set -e

# Function to print errors only
err() { echo "$@" 1>&2; }

# Check if git-cliff is installed
if ! command -v git-cliff &> /dev/null; then
    echo "git-cliff is not installed. Please install it first."
    exit 1
fi

# Check if cargo-verset is installed
if ! command -v cargo-verset &> /dev/null; then
    err "cargo-verset is not installed. Please install it with:"
    err "    cargo install cargo-verset"
    exit 1
fi

# Get the bumped version from git-cliff
version=$(git-cliff --bumped-version)
current_version=$(git describe --tags)

# if the version is the same as the current version, exit
if [ "$version" == "$current_version" ]; then
    echo "Version $version is already the current version. No changes made."
    exit 0
fi

echo "Calculated version: $version"
echo "Updating version in Cargo.toml..."
cargo verset package -v "$version"
cargo verset dependency -n get-size-derive2 -v "$version"
echo "Version updated successfully in Cargo.toml."

# Generate the changelog
echo "Generating changelog..."
git cliff --output CHANGELOG.md -t "$version"
echo "Changelog generated successfully."
# Ask for confirmation before committing
echo "Do you want to commit the changes? (y/n)"
read -r answer
if [ "$answer" != "y" ]; then
    echo "Changes not committed. Exiting."
    exit 0
fi

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "release($version)"
git tag -a "$version" -m "Release $version"

echo "Changes committed and tagged with version $version."
echo "Don't forget to push the changes to the remote repository."