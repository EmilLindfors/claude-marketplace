---
description: Add a new entry to the CHANGELOG.md file following Keep a Changelog format
---

Add a new changelog entry to the project's CHANGELOG.md file. This command helps you document changes following the Keep a Changelog format (https://keepachangelog.com/).

## Context
This project maintains a CHANGELOG.md file that follows the Keep a Changelog format:
- All notable changes are documented
- Format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- The project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
- Entries are organized by version with date stamps
- Changes are categorized as: Added, Changed, Deprecated, Removed, Fixed, Security

## Task
Help the user add a well-formatted changelog entry:

1. **Check for CHANGELOG.md**: Look for CHANGELOG.md in the project root
2. **Identify the [Unreleased] section**: New entries go under the [Unreleased] section
3. **Ask the user for details**:
   - What type of change is this? (Added/Changed/Fixed/Removed/Security/Deprecated)
   - What is the description of the change?
   - Any additional context or details?
4. **Format the entry** following the Keep a Changelog format:
   - Use proper heading level (### for category)
   - Use bullet points (-)
   - Include relevant technical details
   - Reference related files, endpoints, or components
   - Be concise but descriptive
5. **Add the entry** under the appropriate category in the [Unreleased] section
6. **Stage the file**: Run `git add CHANGELOG.md` to stage the changes
7. **Confirm**: Show the user the added entry and confirm it's been staged

## Keep a Changelog Categories

### Added
For new features, endpoints, functionality, or capabilities.

### Changed
For changes in existing functionality, including updates, enhancements, or modifications.

### Deprecated
For soon-to-be removed features or functionality.

### Removed
For removed features, endpoints, or functionality.

### Fixed
For bug fixes, error corrections, or issue resolutions.

### Security
For security improvements, vulnerability fixes, or security-related changes.

## Entry Format Guidelines

### Good Examples
```markdown
### Added
- **System Metrics Collection**: Comprehensive system monitoring using dedicated PyIceberg table
  - Multi-Service Support: Service identification with hostname and environment
  - Comprehensive Metrics: CPU usage, memory, disk, network, and process metrics
  - Partitioned Storage: Efficiently partitioned by service_name, date, and hour

### Fixed
- **Loss Mortality Endpoint**: Fixed window function partitioning bug in `/v3/mortality/areas/month` endpoint
  - Root Cause: Window functions were using incorrect partition clause
  - Impact: Cumulative calculations now update correctly with date parameter changes
  - Solution: Updated partition clause to `PARTITION BY fdir.aquacloud_area_name`

### Changed
- **Parameter Standardization**: Migrated all v3 endpoints from `include_self` to `exclude_self` parameter
  - Updated 13 v3 SQL queries to use `$exclude_self` template parameter
  - Behavioral Consistency: Both v2 and v3 APIs now use identical parameter naming
```

### Formatting Tips
- Use **bold** for main component or feature names
- Use sub-bullets for technical details, root causes, solutions, impacts
- Include file paths for code changes (e.g., `services/v3/api/router.py`)
- Reference endpoint paths when applicable (e.g., `/v3/mortality/areas/month`)
- Be specific and technical - developers will read this
- Group related changes under one main bullet when appropriate

## Example Workflow

```
User: I fixed a bug where the API was returning 500 errors on the feeding endpoint

You: I'll help you add that to the changelog. Let me check the CHANGELOG.md file first.

[After reading the file]

Let me add this under the Fixed section in the [Unreleased] area:

### Fixed
- **Feeding Endpoint Error**: Fixed 500 server error in `/v3/feeding/sfr-by-weeknumber-and-year` endpoint
  - Root Cause: Missing AND clause in SQL WHERE statement
  - Solution: Added proper SQL syntax to fix query parsing
  - Impact: Endpoint now returns data successfully

Does this accurately describe your fix? Would you like me to add any additional details?
```

## Important Notes
- Always add entries to the **[Unreleased]** section
- Don't modify versioned sections (those are historical records)
- If [Unreleased] section doesn't exist, create it at the top after the header
- Maintain consistent formatting with existing entries
- Use proper markdown syntax
- Stage the file after adding the entry so it's ready for commit
- Multiple related changes can be grouped under one main bullet point with sub-bullets

Remember: Good changelog entries help developers understand what changed, why it changed, and what impact it has.
