# AQC Changelog Plugin - Context & Best Practices

This document provides detailed context about the changelog plugin, its design philosophy, and best practices for maintaining high-quality changelogs.

## Plugin Philosophy

The AQC Changelog plugin is built on these core principles:

1. **Enforcement First**: Prevention is better than cure - the hook ensures changelogs stay updated
2. **Developer-Friendly**: Clear error messages and helpful commands make it easy to do the right thing
3. **Standards-Based**: Follows Keep a Changelog and Semantic Versioning standards
4. **Flexible but Consistent**: Supports different styles while maintaining format consistency
5. **Automation-Friendly**: Git hooks and CLI tools can integrate seamlessly

## Keep a Changelog Format

The plugin enforces the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format:

### Standard Structure

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New features go here

### Changed
- Changes to existing features

### Deprecated
- Soon-to-be removed features

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security improvements

## [1.0.0] - 2025-01-01

### Added
- Initial release
```

## Understanding the Hook

### How It Works

The `check-changelog-before-commit.py` hook:

1. **Listens** for `UserPromptSubmit` events in Claude Code
2. **Detects** when the user is attempting a git commit
3. **Checks** if CHANGELOG.md has been modified (staged or unstaged)
4. **Blocks** the commit if no changelog update is found
5. **Provides** helpful error message with instructions

### Detection Patterns

The hook detects commit attempts using these patterns:
- `git commit`
- `commit changes`
- `create a commit`
- `make a commit`
- `commit ... to git`

### Changelog Detection

The hook checks for modifications in:
- `CHANGELOG.md` (case variations: CHANGELOG.MD, changelog.md)
- Both staged changes (`git diff --cached`)
- Unstaged changes (`git diff`)

### Fail-Safe Design

If the hook encounters errors:
- **Exits with code 0** (success) to not block the user
- **Logs error to stderr** for debugging
- **Assumes best intent** - doesn't prevent work

## Writing Great Changelog Entries

### The Anatomy of a Good Entry

A well-written changelog entry should answer:
1. **What changed?** - Clear description of the change
2. **Why did it change?** - Context, root cause, or reason
3. **What's the impact?** - How does this affect users/developers?
4. **Where did it change?** - Files, components, endpoints affected

### Entry Templates by Category

#### Added - New Features

```markdown
### Added
- **Feature Name**: Brief description of the new capability
  - **Component 1**: Specific detail about first component
  - **Component 2**: Specific detail about second component
  - **Configuration**: Environment variables or settings
  - **Documentation**: Any new docs or guides
```

**Example**:
```markdown
### Added
- **MQTT Integration**: Full MQTT publish/subscribe capabilities for real-time messaging
  - **TCP MQTT Client**: Reliable server-to-server MQTT communication on port 1883
  - **WebSocket Streaming**: Real-time message broadcasting to web clients
  - **JWT Authentication**: MQTT client automatically uses JWT tokens from authenticated requests
  - **Configuration**: Flexible broker configuration via environment variables
```

#### Changed - Modifications

```markdown
### Changed
- **Component Name**: Description of what changed
  - **Previous Behavior**: What it did before
  - **New Behavior**: What it does now
  - **Reason**: Why the change was made
  - **Migration**: How to adapt to the change (if breaking)
```

**Example**:
```markdown
### Changed
- **BREAKING CHANGE**: All v3 API endpoints now require admin authentication
  - **Previous Behavior**: Basic user authentication was sufficient
  - **New Behavior**: Admin role required for all v3 endpoints
  - **Affected Endpoints**: `/v3/common/*`, `/v3/feeding/*`, `/v3/loss_mortality/*`
  - **Reason**: Enhanced security and access control for production data
  - **Migration**: Users must have admin role to access v3 endpoints
```

#### Fixed - Bug Fixes

```markdown
### Fixed
- **Issue Description**: Clear description of the bug
  - **Root Cause**: What was causing the problem
  - **Solution**: How it was fixed
  - **Impact**: What now works correctly
  - **Files**: Files that were modified
```

**Example**:
```markdown
### Fixed
- **Loss Mortality Endpoint**: Fixed window function partitioning bug in `/v3/mortality/areas/month`
  - **Root Cause**: Window functions using incorrect partition clause `PARTITION BY aquacloud_area_name`
  - **Solution**: Updated to use correct partition `PARTITION BY fdir.aquacloud_area_name`
  - **Impact**: Cumulative calculations now update correctly with date parameter changes
  - **Fixed Functions**: `fdir_cumulative_loss_rate_12_months`, `fdir_cumulative_loss_rate_6_months`, `fdir_cumulative_loss_rate_3_months`
  - **Files**: `services/v3/loss_mortality/queries/get_loss_and_mortality_by_area_and_month_fdir.sql`
```

#### Removed - Deletions

```markdown
### Removed
- **Feature/Component**: What was removed
  - **Reason**: Why it was removed
  - **Replacement**: What to use instead (if applicable)
  - **Deprecation**: When it was deprecated (if applicable)
```

**Example**:
```markdown
### Removed
- **Legacy MQTT Metrics System**: Completely removed old MQTT-based telemetry
  - **Removed Components**: MetricsMiddleware, MetricsPublisher, `/v3/metrics/*` endpoints
  - **Reason**: Replaced with more efficient Iceberg-based logging system
  - **Replacement**: Use new IcebergLogHandler for metrics collection
  - **Migration**: Update monitoring dashboards to query Iceberg tables instead
```

#### Security - Security Fixes

```markdown
### Security
- **Security Issue**: Description of vulnerability or improvement
  - **Vulnerability**: What was insecure
  - **Fix**: How it was secured
  - **Impact**: Security benefit
```

**Example**:
```markdown
### Security
- **Token Expiry Handling**: Enhanced authentication with automatic token refresh
  - **Vulnerability**: Expired tokens causing unauthorized access errors
  - **Fix**: Proactive token refresh when tokens are close to expiring (2-minute threshold)
  - **Impact**: Reduced authentication errors and improved session security
```

#### Deprecated - Deprecation Notices

```markdown
### Deprecated
- **Feature Name**: What is being deprecated
  - **Deprecation Date**: When it was deprecated
  - **Removal Date**: When it will be removed (if known)
  - **Alternative**: What to use instead
  - **Migration Guide**: How to transition away
```

## Formatting Best Practices

### Use of Bold and Emphasis

- **Bold** for component names, feature names, issue titles
- `Inline code` for file paths, function names, variables, endpoints
- Normal text for descriptions and explanations

### Hierarchical Structure

```markdown
### Category
- **Main Component**: High-level description
  - **Sub-component**: Specific detail
    - Detail level 3 (use sparingly)
```

### Lists and Bullets

- Use `-` for bullet points (not `*` or `+`)
- Maintain consistent indentation (2 spaces per level)
- Group related items under one main bullet
- Keep parallel structure in lists

### Technical References

Include specific technical details:
- **File paths**: `services/v3/feeding/service.py`
- **Endpoints**: `/v3/feeding/sfr-by-weeknumber-and-year`
- **Functions**: `calculate_sfr()`, `get_mortality_rate()`
- **SQL queries**: `services/v3/feeding/queries/get_sfr_by_week.sql`
- **Environment variables**: `DB_CACHE_SIZE_MB=2048`
- **Commit SHAs**: Use short form `b0d9352` in entries

### Line Length and Wrapping

- No hard line length limit
- Break lines naturally at punctuation
- Keep sub-bullets readable
- Don't break in the middle of technical references

## Semantic Versioning

The plugin supports [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

### Version Format: MAJOR.MINOR.PATCH

- **MAJOR**: Breaking changes, incompatible API changes
- **MINOR**: New features, backward-compatible
- **PATCH**: Bug fixes, backward-compatible

### Version Sections in Changelog

```markdown
## [Unreleased]
<!-- Ongoing work, not yet released -->

## [2.1.0] - 2025-01-15
<!-- Latest release -->

## [2.0.0] - 2025-01-01
<!-- Previous release -->
```

### When to Update Versions

- **Don't** manually update version sections - use release tools
- **Do** add entries to [Unreleased]
- **Let** release automation move entries from [Unreleased] to versioned sections

## Plugin Commands Deep Dive

### /changelog-add

**When to use**:
- After making code changes
- Before committing
- When fixing bugs
- When adding features

**Workflow**:
1. Command prompts for change type
2. User provides description
3. Agent formats entry properly
4. Entry added to [Unreleased]
5. File automatically staged

**Tips**:
- Be specific about what changed
- Include file paths when relevant
- Mention breaking changes explicitly
- Group related changes together

### /changelog-view

**When to use**:
- Checking what's in [Unreleased]
- Reviewing recent changes
- Understanding changelog format
- Searching for specific changes

**Workflow**:
1. Reads CHANGELOG.md
2. Formats and displays recent entries
3. Highlights structure and categories
4. Offers guidance on format

**Tips**:
- Use to learn the project's changelog style
- Check before adding new entries
- Verify your changes are documented

### /changelog-init

**When to use**:
- Starting a new project
- Adding changelog to existing project without one
- Resetting changelog structure

**Workflow**:
1. Checks for existing CHANGELOG.md
2. Detects project version from package files
3. Creates properly formatted file
4. Adds helpful comments and examples

**Tips**:
- Run this once per project
- Customize the initial version
- Review and adjust template before first commit

## Changelog Writer Agent

The `changelog-writer` agent is a specialized tool for complex changelog tasks.

### When to Use the Agent

Use the agent when:
- Writing complex, multi-faceted changes
- Documenting large refactors
- Creating detailed bug fix entries
- Grouping many related changes
- You want expert formatting assistance

### Agent Capabilities

The agent can:
- Research changes from git diff
- Read modified files to understand impact
- Write technically detailed entries
- Follow project-specific style
- Group related changes logically
- Provide context and reasoning

### Example Agent Invocation

```
@changelog-writer I need to document a large refactoring where I:
- Renamed 15 functions across 8 files
- Updated SQL queries to use new naming
- Modified 3 API endpoints
- Updated tests

The reason was to improve code consistency and readability.
```

The agent will:
1. Review the git diff
2. Read affected files
3. Check existing changelog style
4. Write comprehensive entry with all details
5. Group changes logically
6. Stage the changelog file

## Common Patterns and Examples

### Pattern: Multi-Component Feature Addition

```markdown
### Added
- **System Metrics Collection**: Comprehensive monitoring infrastructure
  - **PyIceberg Integration**: Dedicated metrics table with partitioning
  - **Multi-Service Support**: Service identification and environment tracking
  - **Comprehensive Metrics**: CPU, memory, disk, network, and process metrics
  - **Retry Mechanism**: Exponential backoff with jitter for reliability
  - **Configuration**: Environment variables for customization
  - **Cross-Platform**: Windows and Unix/Linux support
```

### Pattern: Breaking Change Documentation

```markdown
### Changed
- **BREAKING CHANGE**: Schema system completely redesigned
  - **Previous System**: Hardcoded `{MART_SCHEMA}` placeholders
  - **New System**: Environment-based layer placeholders `{{mart}}`
  - **Impact**: All SQL queries must be updated to new format
  - **Migration**: Replace `{MART_SCHEMA}` with `{{mart}}` in queries
  - **Benefit**: Simplified configuration, no race conditions
  - **Files**: All files in `services/**/queries/*.sql` (200+ files)
```

### Pattern: Complex Bug Fix

```markdown
### Fixed
- **Docker Build and Deployment Pipeline**: Fixed multiple critical issues
  - **Build Timeouts**:
    - Root Cause: Unnecessary debugging tools in Dockerfile
    - Solution: Removed unused packages, added apt-get retry logic
    - Commit: `b0d9352`
  - **ECR Push Issues**:
    - Root Cause: Improper version string sanitization (contained `+`)
    - Solution: Replace `+` with `-` in Docker tags
    - Commit: `a24d793`
  - **CI/CD Pipeline**:
    - Root Cause: Just tool not installed before use
    - Solution: Install just before push step
    - Commit: `dd67440`
```

### Pattern: Infrastructure Improvements

```markdown
### Added
- **Comprehensive GitHub Actions Workflow Suite**:
  - **PR Validation**: Version checks, Docker build validation, security scanning
  - **Release Workflow**: Automated GitHub releases and production deployment
  - **Manual Deployment**: Environment-specific deployment with protection
  - **Type Checking**: Pyrefly integration for type validation
  - **Post-Deployment Testing**: Automated API tests after staging deployments
```

## Integration with Git Workflow

### Recommended Git Workflow

1. **Make changes** to code
2. **Add changelog entry** immediately (don't wait until commit)
3. **Stage changes**: `git add .` (includes CHANGELOG.md)
4. **Commit**: Claude Code hook validates changelog was updated
5. **Push**: Changes with documented changelog

### Handling Multiple Commits

For multiple related commits:
- **Option 1**: Add one comprehensive entry covering all commits
- **Option 2**: Add entries for each commit, then consolidate before release

### Handling Branches

- Keep [Unreleased] updated in feature branches
- Merge conflicts in changelog are rare but should be resolved carefully
- On merge, [Unreleased] accumulates entries from all branches

### Release Process

When releasing a version:
1. Review [Unreleased] section
2. Create new version section (e.g., `## [1.2.0] - 2025-01-15`)
3. Move [Unreleased] entries to the new version section
4. Leave [Unreleased] empty or with a blank template
5. Tag the release in git

## Troubleshooting Common Issues

### Hook Not Triggering

**Symptom**: Can commit without updating changelog

**Solutions**:
- Check hook is registered in `.claude/settings.json`
- Verify Python 3 is installed: `python3 --version`
- Make script executable: `chmod +x plugins/aqc-changelog/hooks/check-changelog-before-commit.py`
- Check for syntax errors in hook script

### Hook Blocking Valid Commits

**Symptom**: Hook blocks even when changelog is updated

**Solutions**:
- Ensure CHANGELOG.md is staged: `git add CHANGELOG.md`
- Check file name case (must be CHANGELOG.md, not changelog.md)
- Verify actual changes exist in the file
- Check git detects the changes: `git diff --cached CHANGELOG.md`

### Commands Not Available

**Symptom**: `/changelog-add` command not found

**Solutions**:
- Verify plugin is registered in `.claude-plugin/marketplace.json`
- Check `plugin.json` exists and is valid
- Restart Claude Code to reload plugins
- Check command files exist in `commands/` directory

### Agent Not Responding

**Symptom**: `@changelog-writer` agent doesn't respond

**Solutions**:
- Verify agent file exists: `agents/changelog-writer.md`
- Check agent is referenced in `plugin.json`
- Try invoking with full context
- Check Claude Code logs for errors

## Advanced Usage

### Custom Hook Configuration

You can customize the hook behavior by editing `check-changelog-before-commit.py`:

```python
# Customize commit patterns to detect
commit_patterns = [
    r'\bgit\s+commit\b',
    r'\bcommit\s+(the\s+)?changes?\b',
    # Add custom patterns here
]

# Customize changelog file names
changelog_files = ['CHANGELOG.md', 'HISTORY.md', 'CHANGES.md']
```

### Multiple Changelog Files

If your project uses multiple changelogs:

1. Modify the hook to check all files
2. Update commands to target specific changelog
3. Use different categories for different components

### Automated Changelog Generation

Integrate with tools like:
- `conventional-changelog` for automated entries
- `release-please` for version management
- CI/CD pipelines for release automation

The plugin complements these tools by:
- Enforcing manual review before commit
- Ensuring human-readable entries
- Maintaining consistent format

## Best Practices Summary

1. ✅ **Update changelog as you code**, not before commit
2. ✅ **Be specific and technical** - include file paths, endpoints, functions
3. ✅ **Group related changes** under one main bullet
4. ✅ **Include context** - explain root causes and impacts
5. ✅ **Use consistent formatting** - match existing style
6. ✅ **Reference breaking changes clearly**
7. ✅ **Add to [Unreleased]** - let release tools manage versions
8. ✅ **Stage changelog with code** - don't forget to `git add CHANGELOG.md`
9. ✅ **Use the agent for complex entries** - leverage automation
10. ✅ **Review changelog before releases** - ensure completeness

## Future Enhancements

Potential improvements to the plugin:

- **Auto-detection**: Automatically suggest changelog category based on code changes
- **PR Integration**: Check changelog in PR validation
- **Release Notes**: Generate release notes from changelog
- **Template System**: Project-specific changelog templates
- **Version Management**: Automated version bumping based on changelog
- **Multi-language**: Support for non-English changelogs
- **Changelog Linting**: Validate format and style automatically

## Contributing

To contribute to this plugin:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly (especially the hook)
5. Update documentation (README.md and CONTEXT.md)
6. Submit pull request

---

**Remember**: A well-maintained changelog is documentation, communication, and historical record all in one. Invest the time to do it right! 📝
