#!/usr/bin/env python3
"""
Changelog enforcement hook for Claude Code.

This hook checks if a git commit attempt includes a changelog update.
Blocks commits that don't have corresponding changelog entries.
"""

import json
import sys
import subprocess
import re
from pathlib import Path


def run_git_command(cmd, cwd):
    """Run a git command and return the output."""
    try:
        result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=5)
        return result.stdout.strip(), result.returncode
    except subprocess.TimeoutExpired:
        return "", 1
    except Exception:
        return "", 1


def is_commit_attempt(prompt):
    """Check if the prompt is attempting a git commit."""
    commit_patterns = [
        r"\bgit\s+commit\b",
        r"\bcommit\s+(the\s+)?changes?\b",
        r"\bcreate\s+a\s+commit\b",
        r"\bmake\s+a\s+commit\b",
        r"\bcommit\s+.*\s+to\s+git\b",
    ]

    prompt_lower = prompt.lower()
    return any(re.search(pattern, prompt_lower) for pattern in commit_patterns)


def check_changelog_modified(cwd):
    """Check if CHANGELOG.md has been modified in the current branch."""

    # Check if we're in a git repository
    _, returncode = run_git_command(["git", "rev-parse", "--git-dir"], cwd)
    if returncode != 0:
        return True  # Not in a git repo, allow the commit

    # Find CHANGELOG files
    changelog_files = []
    for pattern in ["CHANGELOG.md", "CHANGELOG.MD", "changelog.md"]:
        changelog_path = Path(cwd) / pattern
        if changelog_path.exists():
            changelog_files.append(pattern)

    if not changelog_files:
        return False  # No changelog file found, require user to address this

    # Check if any changelog file is in staged changes
    staged_output, _ = run_git_command(["git", "diff", "--cached", "--name-only"], cwd)
    for changelog_file in changelog_files:
        if changelog_file in staged_output:
            return True

    # Check if any changelog file has unstaged changes
    unstaged_output, _ = run_git_command(["git", "diff", "--name-only"], cwd)
    for changelog_file in changelog_files:
        if changelog_file in unstaged_output:
            return True

    return False


def check_unused_sections(cwd):
    """Check if changelog has empty/unused sections that should be cleaned up."""
    # Find the changelog file
    changelog_path = None
    for pattern in ["CHANGELOG.md", "CHANGELOG.MD", "changelog.md"]:
        path = Path(cwd) / pattern
        if path.exists():
            changelog_path = path
            break

    if not changelog_path:
        return None  # No changelog found

    try:
        content = changelog_path.read_text(encoding='utf-8')
    except Exception:
        return None  # Can't read file

    # Find the [Unreleased] section
    unreleased_match = re.search(r'## \[Unreleased\](.*?)(?=## \[|\Z)', content, re.DOTALL)
    if not unreleased_match:
        return None  # No unreleased section

    unreleased_section = unreleased_match.group(1)

    # Check for empty category sections
    categories = ['Added', 'Changed', 'Deprecated', 'Removed', 'Fixed', 'Security']
    empty_sections = []

    for category in categories:
        # Look for the category heading
        category_pattern = rf'### {category}\s*\n'
        if re.search(category_pattern, unreleased_section):
            # Check if there's content after the heading (before next heading or end)
            content_pattern = rf'### {category}\s*\n(.*?)(?=###|\Z)'
            match = re.search(content_pattern, unreleased_section, re.DOTALL)
            if match:
                section_content = match.group(1).strip()
                # Filter out HTML comments
                section_content = re.sub(r'<!--.*?-->', '', section_content, flags=re.DOTALL).strip()
                if not section_content:
                    empty_sections.append(category)

    return empty_sections if empty_sections else None


def main():
    """Main hook execution."""
    try:
        # Read input from stdin
        input_data = json.load(sys.stdin)

        prompt = input_data.get("prompt", "")
        cwd = input_data.get("cwd", ".")

        # Check if this is a commit attempt
        if not is_commit_attempt(prompt):
            # Not a commit attempt, allow it
            sys.exit(0)

        # Check if changelog has been modified
        if check_changelog_modified(cwd):
            # Changelog has been updated, now check for unused sections
            empty_sections = check_unused_sections(cwd)
            if empty_sections:
                # Warn about empty sections that should be cleaned up
                sections_list = ", ".join(empty_sections)
                output = {
                    "decision": "block",
                    "reason": f"""⚠️  Changelog cleanup required!

Your CHANGELOG.md has empty sections that should be removed before committing:
{sections_list}

Please remove these empty category headings from the [Unreleased] section to keep the changelog clean.

Only include category headings that have actual entries under them.

💡 Tip: Edit CHANGELOG.md to remove the empty ### headings, then stage the file again.""",
                    "hookSpecificOutput": {
                        "hookEventName": "UserPromptSubmit",
                        "additionalContext": "Empty changelog sections should be removed before committing.",
                    },
                }
                print(json.dumps(output))
                sys.exit(2)

            # Changelog has been updated and is clean, allow the commit
            sys.exit(0)

        # Block the commit - no changelog update found
        output = {
            "decision": "block",
            "reason": """⚠️  Changelog update required!

You're attempting to commit changes without updating the CHANGELOG.md file.

Please:
1. Add an entry to CHANGELOG.md describing your changes
2. Follow the Keep a Changelog format (https://keepachangelog.com/)
3. Add the changes under the [Unreleased] section
4. Stage the changelog file: git add CHANGELOG.md
5. Then retry your commit

💡 Tip: You can use /changelog:changelog-add command to add an entry quickly.""",
            "hookSpecificOutput": {
                "hookEventName": "UserPromptSubmit",
                "additionalContext": "The changelog must be updated before committing code changes.",
            },
        }

        print(json.dumps(output))
        sys.exit(2)  # Exit code 2 indicates blocking

    except Exception as e:
        # If the hook fails, don't block the user (fail open)
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(0)


if __name__ == "__main__":
    main()
