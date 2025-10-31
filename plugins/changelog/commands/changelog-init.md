---
description: Initialize a new CHANGELOG.md file following Keep a Changelog format
---

Create a new CHANGELOG.md file for the project following the Keep a Changelog format standards.

## Context
A CHANGELOG.md file is essential for tracking all notable changes to a project. It should:
- Follow the Keep a Changelog format (https://keepachangelog.com/en/1.0.0/)
- Use Semantic Versioning (https://semver.org/spec/v2.0.0.html)
- Be human-readable and easy to maintain
- Have clear categories for different types of changes

## Task
Initialize a new changelog file with proper structure:

1. **Check if CHANGELOG.md exists**: Don't overwrite an existing changelog
2. **Create the file structure**:
   - Add the standard header and introduction
   - Include an [Unreleased] section for upcoming changes
   - Add a template version section (if the project has an initial version)
   - Include links to Keep a Changelog and Semantic Versioning
3. **Ask about initial version**: Should we add an initial version entry (e.g., [1.0.0])?
4. **Customize for the project**:
   - Check if there's a version number in the project (package.json, pyproject.toml, etc.)
   - Include relevant context for the specific project type
5. **Create the file**: Write the CHANGELOG.md to the project root
6. **Provide guidance**: Show the user how to use the new changelog

## Standard Changelog Template

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - YYYY-MM-DD

### Added
- Initial release
```

## Categories Explanation

Include a comment in the changelog explaining the categories:

```markdown
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
```

## Workflow

```
User: I need to create a changelog for this project

You: I'll help you initialize a new CHANGELOG.md file. Let me first check if one already exists.

[Check for existing CHANGELOG.md]

Great! I don't see an existing CHANGELOG.md. Let me create one for you.

[Check for version information in package files]

I found your project is using version 1.0.0 from pyproject.toml.

I'll create a CHANGELOG.md with:
- Standard Keep a Changelog header
- [Unreleased] section for upcoming changes
- [1.0.0] section for the initial release

[Create the file]

✅ Created CHANGELOG.md in the project root!

Next steps:
1. Review the initial structure
2. Add any existing changes to the [Unreleased] section
3. Use /changelog-add when you make changes to the project
4. The changelog hook will ensure you update the changelog before each commit

Would you like me to add any specific entries to get you started?
```

## Version Detection

Try to detect the current version from common files:
- `package.json` (Node.js): Check the "version" field
- `pyproject.toml` (Python): Check [tool.poetry.version] or [project] version
- `Cargo.toml` (Rust): Check [package] version
- `pom.xml` (Java/Maven): Check <version> tag
- `build.gradle` (Gradle): Check version property
- `setup.py` (Python): Check version parameter
- `__version__.py`: Check __version__ variable

## Customization Options

Ask the user if they want to:
- Include an initial version entry or just [Unreleased]
- Add any specific categories they commonly use
- Include example entries to guide future updates
- Add project-specific notes or conventions

## Important Notes
- Never overwrite an existing CHANGELOG.md without explicit confirmation
- Use the current date (YYYY-MM-DD format) for version entries
- Include helpful comments for first-time users
- Make sure the format is exactly correct (Keep a Changelog is specific about format)
- After creating, suggest adding it to git: `git add CHANGELOG.md`

## Follow-up

After creating the changelog:
1. Show the user the created file
2. Explain how to use it
3. Mention the /changelog-add command
4. Remind them about the changelog hook that will enforce updates

Remember: A good initial changelog structure sets the tone for maintaining it throughout the project's lifetime.
