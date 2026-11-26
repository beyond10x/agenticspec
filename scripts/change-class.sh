#!/bin/sh
# The change class of a list of changed files, one path per line on standard input. The job
# `build` of .github/workflows/pages.yml runs it; scripts/change-class.test.sh holds its cases.
#
#   planning   only .engineering/ and Markdown outside the site's sources: nothing is built
#   docs-site  the site's sources (website/, docs/, research/, ui/README.md) and planning files:
#              the site is built, the code gate is not run
#   code       anything else, or an empty list: the site is built and `task check` runs
#
# Prints the class on one line.
set -eu

planning=0 site=0 code=0
while IFS= read -r f || [ -n "$f" ]; do
  case "$f" in
    '') ;;
    .engineering/*) planning=1 ;;
    website/* | docs/* | research/* | ui/README.md) site=1 ;;
    crates/* | generated/* | spec/* | ui/* | queries/*) code=1 ;;
    *.md) planning=1 ;;
    *) code=1 ;;
  esac
done

if [ "$code" = 1 ] || [ "$planning$site" = 00 ]; then
  echo code
elif [ "$site" = 1 ]; then
  echo docs-site
else
  echo planning
fi
