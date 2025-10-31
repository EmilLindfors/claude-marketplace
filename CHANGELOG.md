# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Changelog Plugin**: Added comprehensive changelog management plugin for Claude Code
  - Format Compliance: Follows Keep a Changelog format and Semantic Versioning standards
  - Three Commands: `/changelog:changelog-init` to create new CHANGELOG.md, `/changelog:changelog-add` to add entries, and `/changelog:changelog-view` to review recent changes
  - Guided Workflow: Interactive prompts help users categorize changes (Added, Changed, Fixed, Removed, Security, Deprecated)
  - Agent Integration: Specialized changelog-writer agent ensures consistent formatting and quality
  - PreToolUse Hook: Automatic hook integration that intercepts git commits to validate changelog updates and ensure empty sections are cleaned up (configured in `plugins/changelog/hooks/hooks.json`)

<!--
Categories for changelog entries:

- Added: New features, endpoints, or functionality
- Changed: Changes in existing functionality
- Deprecated: Soon-to-be removed features
- Removed: Removed features or functionality
- Fixed: Bug fixes and error corrections
- Security: Security improvements and vulnerability fixes

Each entry should be concise but descriptive, include technical details,
and reference relevant files, endpoints, or components.
-->
