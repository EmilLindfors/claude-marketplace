# AQC Changelog Plugin

A comprehensive changelog management plugin for Claude Code that ensures all code commits include proper changelog entries.

## Overview

The AQC Changelog plugin helps maintain a clean, well-documented changelog following the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and [Semantic Versioning](https://semver.org/spec/v2.0.0.html) standards. It prevents commits without changelog updates, provides convenient commands for managing changelog entries, and includes a specialized agent for writing professional changelog entries.

## Features

### 🔒 Changelog Enforcement Hook
- **Automatic validation**: Checks every commit attempt to ensure CHANGELOG.md has been updated
- **Smart detection**: Only blocks commits when code changes are being committed
- **Clear messaging**: Provides helpful error messages with instructions on how to proceed
- **Fail-safe design**: If the hook encounters an error, it won't block the user

### 📝 Slash Commands

#### `/changelog-add`
Add a new entry to the CHANGELOG.md file with guided prompts and format validation.

**Features**:
- Interactive prompts for change type and description
- Automatic formatting following Keep a Changelog standards
- Auto-categorization (Added, Changed, Fixed, Removed, Security, Deprecated)
- Automatic git staging of the updated file
- Format validation and consistency checks

#### `/changelog-view`
View recent entries from the CHANGELOG.md file with formatted output.

**Features**:
- Display unreleased changes
- Show recent versioned releases
- Search for specific entries
- Format validation
- Statistics on change types

#### `/changelog-init`
Initialize a new CHANGELOG.md file with proper structure.

**Features**:
- Creates standard Keep a Changelog format
- Detects project version from package files
- Includes helpful comments and examples
- Customizable initial structure
- Version detection support for multiple project types

### 🤖 Changelog Writer Agent

A specialized agent (`changelog-writer`) that can be invoked for complex changelog writing tasks.

**Capabilities**:
- Research changes from git diff and modified files
- Write detailed, well-formatted changelog entries
- Follow project-specific changelog style
- Provide technical detail and context
- Group related changes logically
- Stage changes automatically

## Installation

The plugin is already installed in this repository. To enable it in other projects:

1. **Install the plugin**:
   ```bash
   # Copy the plugin to your project
   cp -r plugins/aqc-changelog /path/to/your/project/plugins/
   ```

2. **Register in marketplace.json**:
   ```json
   {
     "plugins": [
       {
         "name": "aqc-changelog",
         "source": "./plugins/aqc-changelog",
         "description": "Changelog management plugin",
         "version": "1.0.0",
         "author": {
           "name": "Aquacloud"
         }
       }
     ]
   }
   ```

3. **Hooks are automatically enabled** when the plugin is enabled. No manual configuration needed!

## Usage

### Basic Workflow

1. **Make code changes** in your project
2. **Add changelog entry**:
   ```
   /changelog-add
   ```
   Follow the prompts to add your entry.

3. **Stage your changes**:
   ```bash
   git add .
   ```

4. **Commit** (the hook will verify changelog was updated):
   ```
   Create a commit with message "Add new feature"
   ```

### Quick Tips

- **View recent changes**: `/changelog-view`
- **Initialize new project**: `/changelog-init`
- **Complex entries**: Use the `changelog-writer` agent for detailed, multi-faceted changes
- **Multiple related changes**: Group them under one main bullet with sub-bullets

### Example Changelog Entry

```markdown
### Added
- **System Metrics Collection**: Comprehensive system monitoring using dedicated PyIceberg table
  - **Multi-Service Support**: Service identification with hostname and environment
  - **Comprehensive Metrics**: CPU, memory, disk, network, and process metrics
  - **Partitioned Storage**: Efficiently partitioned by service_name, date, and hour
  - **Configurable**: Environment variables for intervals and enable/disable control
```

## Changelog Categories

- **Added**: New features, endpoints, or functionality
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features or functionality
- **Fixed**: Bug fixes and error corrections
- **Security**: Security improvements and vulnerability fixes

## Hook Configuration

The changelog hook is automatically configured when the plugin is enabled. It uses the `PreToolUse` hook to intercept git commit commands and validate that:

1. The CHANGELOG.md file has been modified when committing code changes
2. All empty category sections are removed before committing

**Plugin hooks configuration** (in `plugins/changelog/hooks/hooks.json`):
```json
{
  "description": "Changelog validation before commits",
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "python3 ${CLAUDE_PLUGIN_ROOT}/hooks/check-changelog-before-commit.py",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

The hook runs automatically before any Bash tool executions, specifically targeting git commit operations.

## Troubleshooting

### Hook not triggering
- Check that the hook is registered in your settings.json
- Verify the hook script is executable: `chmod +x plugins/aqc-changelog/hooks/check-changelog-before-commit.py`
- Check Python 3 is available: `python3 --version`

### Hook blocking incorrectly
- Ensure CHANGELOG.md is staged: `git add CHANGELOG.md`
- Check that changes are actually in the changelog file
- Verify the changelog file name is correct (case-sensitive)

### Commands not available
- Check the plugin is registered in `.claude-plugin/marketplace.json`
- Verify the plugin.json file exists in the plugin directory
- Restart Claude Code to reload plugins

## Development

### Project Structure

```
plugins/aqc-changelog/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata
├── commands/
│   ├── changelog-add.md     # Add entry command
│   ├── changelog-view.md    # View entries command
│   └── changelog-init.md    # Initialize changelog command
├── agents/
│   └── changelog-writer.md  # Specialized changelog writing agent
├── hooks/
│   └── check-changelog-before-commit.py  # Enforcement hook
├── CONTEXT.md               # Plugin context and best practices
└── README.md               # This file
```

### Testing the Hook

To test the hook manually:

```bash
# Create a test input
echo '{"prompt": "git commit -m test", "cwd": "."}' | \
  python3 plugins/aqc-changelog/hooks/check-changelog-before-commit.py
```

Expected output when changelog is not updated:
```json
{
  "decision": "block",
  "reason": "⚠️ Changelog update required!...",
  "hookSpecificOutput": {...}
}
```

## Best Practices

1. **Update changelog before committing**: Add entries as you make changes, not at the end
2. **Be specific and technical**: Include file paths, endpoint names, function names
3. **Group related changes**: Multiple related fixes can go under one main bullet
4. **Include context**: Explain the "why" behind changes, especially for bug fixes
5. **Use consistent formatting**: Match the existing changelog style
6. **Reference breaking changes**: Clearly mark any breaking changes
7. **Keep it current**: Always add to [Unreleased], versions are managed separately

## Contributing

To contribute improvements to this plugin:

1. Make changes to plugin files
2. Update the version in `plugin.json`
3. Update this README if adding features
4. Test the hook and commands thoroughly
5. Update CONTEXT.md with any new patterns or practices

## License

This plugin is part of the AquaCloud Claude Code plugin collection.

## Support

For issues or questions:
- Check the troubleshooting section above
- Review CONTEXT.md for detailed usage patterns
- Contact the AquaCloud team at support@aquacloud.ai

---

**Remember**: A well-maintained changelog is a gift to your future self and your team! 📝✨
