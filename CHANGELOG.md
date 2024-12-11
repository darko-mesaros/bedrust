# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- TODO: Ability to generate images
- TODO: Better code testing
- TODO: Handle longer pastes better

## [0.8.4] - 2024-12-11

### Changed
- Added support for the latest Amazon Nova models
- Added support (finally) for Claude 3 Opus
- You now get an error when your call is being throttled (Service quota issues)
- Cleaned up some stale code and removed commented out lines that serve no purpose

## [0.8.3] - 2024-11-14

### Fixed
- There was an issue where the config checker was still looking for the model config file

## [0.8.2] - 2024-11-07
### Added
- BETA: You can export conversations as HTML now

### Changed
- Cleaned up the saving and recalling process
- Moved out the HTML source code to an external file

## [0.8.1] - 2024-11-06
### Added
- BETA: You can now save and recall conversations

### Changed
- Support for additional models (Claude 3.5 and v2, Haiku 3.5, Llama 3.1 models)
- Better system prompts
- Cleaned up some code and improved way things are orchestrated

## [0.8.0] - 2024-09-14
### Added
- Support for Mistral Large 2

### Changed
- Changed the whole underlying system to use the AWS Bedrock Converse API. No longer having the need to pass model specific parameters depending on the model chosen.
- Updated the code chat feature to support `.gitignore` files
- Added additional retries and checks to the code chat feature

### Thanks
- Thank you Stephen for the feature feedbacks

## [0.7.5] - 2024-07-30

### Added
- Added support for Claude 3.5 Sonnet
- Added a *BETA* feature that allows you to put a code repository into context so a user can chat about their code.

### Changed
- Fixed the region provider issue where it would always default to the `us-west-2` region, no matter the profile being used.

## [0.7.4] - 2024-05-06
### Changed
- Fixed the credential provider chain issue. Now it will try a few credential providers, before giving up.

### Thanks
- Thanks to kaumnen and electric__universe on Twitch <3 

### Added
## [0.7.3] - 2024-04-20
- Added support for the Mistral Large model
- Special command to clear current chat history
- Added configuration option to hide the ASCII banner

### Changed
- Updated the `bedrust_config.ron` file with some comments
- Updated the package versions

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
