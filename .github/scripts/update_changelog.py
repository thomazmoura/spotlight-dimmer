#!/usr/bin/env python3
"""
Update changelog after release.

This script moves unreleased content to a new version section
and creates an empty unreleased section for future changes.
"""

import re
import sys
import datetime
import os


def update_changelog(version, changelog_path="CHANGELOG.md"):
    """Update changelog to move unreleased content to versioned section."""
    today = datetime.date.today().strftime("%Y-%m-%d")

    try:
        with open(changelog_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Replace [Unreleased] section with new version and empty unreleased section
        unreleased_pattern = r'## \[Unreleased\]\s*(.*?)\s*(?=## \[)'

        def replace_unreleased(match):
            unreleased_content = match.group(1).strip()
            if unreleased_content:
                return "## [Unreleased]\n\n## [{}] - {}\n\n{}\n\n".format(
                    version, today, unreleased_content
                )
            else:
                return "## [Unreleased]\n\n## [{}] - {}\n\n".format(version, today)

        updated_content = re.sub(unreleased_pattern, replace_unreleased, content, flags=re.DOTALL)

        # If no unreleased section found, add one at the top
        if updated_content == content:
            # Find the first version section and insert before it
            version_pattern = r'(## \[)'
            updated_content = re.sub(
                version_pattern,
                '## [Unreleased]\\n\\n## [{}] - {}\\n\\n\\1'.format(version, today),
                content,
                count=1
            )

        # Write the updated content back
        with open(changelog_path, 'w', encoding='utf-8') as f:
            f.write(updated_content)

        print("SUCCESS: Updated changelog for version {}".format(version))
        return True

    except Exception as e:
        print("ERROR: Error updating changelog: {}".format(e))
        return False


def main():
    """Main execution function."""
    # Get version from command line argument or environment variable
    version = None

    if len(sys.argv) > 1:
        version = sys.argv[1]
    elif os.getenv('VERSION'):
        version = os.getenv('VERSION')
    else:
        print("ERROR: Version not provided. Use as argument or VERSION environment variable.")
        sys.exit(1)

    # Remove 'v' prefix if present
    if version.startswith('v'):
        version = version[1:]

    print(f"Updating changelog for version: {version}")

    success = update_changelog(version)

    if not success:
        sys.exit(1)


if __name__ == "__main__":
    main()