---
description: View recent entries from the CHANGELOG.md file
---

Display recent changelog entries from the project's CHANGELOG.md file to help understand recent changes and the changelog format.

## Context
The CHANGELOG.md file contains all notable changes to the project, organized by version and date following the Keep a Changelog format.

## Task
Help the user view and understand recent changelog entries:

1. **Locate CHANGELOG.md**: Find the CHANGELOG.md file in the project root or nearby repositories
2. **Read the file**: Load the changelog content
3. **Display recent entries**:
   - Show the [Unreleased] section if it exists (this shows pending changes)
   - Show the 3-5 most recent versioned releases
   - Format the output in a readable way
4. **Provide context**:
   - Highlight the changelog format and structure
   - Point out different change categories (Added, Changed, Fixed, etc.)
   - Show examples of well-formatted entries
5. **Offer next steps**:
   - Suggest using /changelog-add if they need to add an entry
   - Point out where new entries should be added

## Display Format

Show the changelog content in a structured way:

```markdown
# Recent Changelog Entries

## [Unreleased]
[If there are unreleased entries, show them here]

## [Version] - Date
[Show recent version entries]

---

💡 Tips:
- New entries should be added to the [Unreleased] section
- Use /changelog-add to add a new entry
- Follow the existing format and style
```

## Example Output

```
# Recent Changelog Entries from CHANGELOG.md

## [Unreleased]

### Fixed
- **Feeding Endpoint Error**: Fixed 500 server error in `/v3/feeding/sfr-by-weeknumber-and-year` endpoint

## [3.7.19] - 2025-09-26

## [3.7.18] - 2025-09-26

## [3.7.17] - 2025-09-25

### Changed
- Use sudo apt install instead of curl for just install

### Fixed
- Reintroduced the period param

---

💡 The changelog follows Keep a Changelog format (https://keepachangelog.com/)

Categories used:
- Added: New features
- Changed: Changes in existing functionality
- Fixed: Bug fixes
- Removed: Removed features
- Security: Security improvements
- Deprecated: Soon-to-be removed features

To add a new entry: /changelog-add
```

## Options

If the user specifies what they want to see, adjust the output:
- "latest version" → Show only the most recent release
- "unreleased" → Show only the [Unreleased] section
- "all" → Show the entire changelog
- "last N versions" → Show the last N versions

## Additional Features

- **Search**: If user asks to search for specific terms, grep through the changelog
- **Statistics**: Can provide counts of different types of changes
- **Format check**: Can validate that the changelog follows the expected format
- **Compare**: Can compare what's in [Unreleased] vs what's committed

## Important Notes
- Make the output readable and well-formatted
- Use appropriate markdown formatting
- Highlight important sections
- Provide helpful context about the changelog structure
- Offer actionable next steps

Remember: This command helps users understand the project's change history and learn the changelog format.
