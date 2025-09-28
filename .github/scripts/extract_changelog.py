#!/usr/bin/env python3
"""
Extract changelog content for GitHub releases.

This script extracts the [Unreleased] section from CHANGELOG.md
and prepares it for use in GitHub release notes.
"""

import re
import sys
import os
from pathlib import Path


def extract_unreleased_content(changelog_path="CHANGELOG.md"):
    """Extract unreleased content from changelog file."""
    try:
        with open(changelog_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Find the unreleased section
        unreleased_match = re.search(
            r'## \[Unreleased\]\s*(.*?)\s*(?=## \[|$)',
            content,
            re.DOTALL
        )

        if unreleased_match:
            unreleased_content = unreleased_match.group(1).strip()
            if unreleased_content:
                print("SUCCESS: Found unreleased content in changelog")
                return unreleased_content
            else:
                print("WARNING: Unreleased section is empty")
                return None
        else:
            print("WARNING: No unreleased section found in changelog")
            return None

    except FileNotFoundError:
        print(f"ERROR: Changelog file not found: {changelog_path}")
        return None
    except Exception as e:
        print(f"ERROR: Error reading changelog: {e}")
        return None


def write_release_notes(content, output_file="release_notes.txt"):
    """Write release notes to output file."""
    try:
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"SUCCESS: Release notes written to {output_file}")
        return True
    except Exception as e:
        print(f"ERROR: Error writing release notes: {e}")
        return False


def main():
    """Main execution function."""
    # Default fallback content
    default_content = "This release includes various improvements and bug fixes."

    # Extract content from changelog
    unreleased_content = extract_unreleased_content()

    # Use extracted content or fallback
    release_content = unreleased_content if unreleased_content else default_content

    # Write to output file
    success = write_release_notes(release_content)

    if success:
        print("=" * 50)
        print("RELEASE NOTES CONTENT:")
        print("=" * 50)
        print(release_content)
        print("=" * 50)

        # Set GitHub Actions output if running in CI
        if os.getenv('GITHUB_OUTPUT'):
            try:
                with open(os.getenv('GITHUB_OUTPUT'), 'a', encoding='utf-8') as f:
                    f.write(f"release-notes<<EOF\n{release_content}\nEOF\n")
                print("SUCCESS: GitHub Actions output set successfully")
            except Exception as e:
                print(f"ERROR: Error setting GitHub Actions output: {e}")
                sys.exit(1)
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()