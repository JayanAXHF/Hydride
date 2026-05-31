#!/usr/bin/env sh

set -eu

: "${WEBHOOK_URL:?WEBHOOK_URL is required}"
: "${MESSAGE_ID:?MESSAGE_ID is required}"

cat > cltool/cltool.toml <<EOF
# Sample configuration for cltool.
#
# Point `git.repo_path` at the repository you want to scan.
# Set `cliff.config_path` to the git-cliff template/config file.
# Replace the Discord values with the real webhook URL and message id(s).

[git]
repo_path = "./"
# Use a literal rev range for a bounded history window.
# The tool will treat HEAD~N..HEAD as "latest N commits".
range = "HEAD~10..HEAD"

[cliff]
config_path = "../cliff.toml"

[discord]
webhook_url = "${WEBHOOK_URL}"
root_message_id = "${MESSAGE_ID}"
overflow_message_ids = []

[output]
max_content_chars = 1900
heading = "# Changelog"
EOF
