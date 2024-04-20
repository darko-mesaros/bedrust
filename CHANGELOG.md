# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- TODO: Ability to generate images
- TODO: Better code testing
- TODO: Handle longer pastes better

## [0.7.3] - 2024-04-20
### Added
- Added support for the Mistral Large model

## [0.7.2] - 2024-04-12
### Added
- Added the ability to pick any available model from a list when running bedrust if there is no default model configured or if you are running `--init`.

### Changed
- Quality of life improvements, and code cleanup
- Better handling of AWS SDK errors

## [0.7.1] - 2024-03-31
### Changed
- Updated README to include additional package requirements for compiling.

## [0.7.0] - 2024-03-30
### Added
- Inital version of this crate that has the abiliuty to be hosted on [crates.io](https://crates.io)
- Ability to create and use config files in users `$HOME/.config/bedrust/` directory
