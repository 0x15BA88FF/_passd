# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Types of changes

  - **Added**: New features.
  - **Changed**: Modifications to existing functionality.
  - **Deprecated**: Features that will be removed in the future.
  - **Removed**: Features that have been removed.
  - **Fixed**: Bug fixes.
  - **Security**: Addressed vulnerabilities.

---

## [Unreleased]

### Added

- Add `remove_directory` command api interface.
- Add `create_directory` command api interface.
- Add `Initialize` command api interface.
- Create a simple http server to handler commands.
- Add passd `decrypt_string` util.
- Add passd `encrypt_string` util.
- Add passd `list_items` command.
- Add passd `find_items` command.
- Add passd `generate_password` command.
- Add passd `move_item` command.
- Add `force` overwrite parameter to `copy` command.
- Add passd `copy` command.
- Add passd `remove_file` command.
- Add passd `remove_directory` command.
- Add passd `create_directory` command.
- Add passd `init` command.
- Create CHANGELOG.md file.
- Create BSD 2-Clause license.
- Initialize repository.

### Changed

- Move `src/utils/decrypt` utility to `src/commands/decrypt`.
- Move `src/utils/encrypt` utility to `src/commands/encrypt`.
- Rename `init` command to `Initialize`.
- Update `init` `pgp_keys` parameter type from `String` to `&str`.
- Update `copy` command function name from `copy` to `copy_item`.
- Update `init` `path` parameter type from `path::PathBuf` to `&path::Path`.

### Removed

- Remove generate password `Filter::All` enum variant.

[unreleased]: https://github.com/0x15BA88FF/passd/compare/main%40%7B1day%7D...main
