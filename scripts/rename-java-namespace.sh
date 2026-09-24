#!/usr/bin/env bash
# issue #82: Java namespace org.ubnfc -> org.unlaxer.ubnfc (the owner holds org.unlaxer on Maven Central,
# not org.ubnfc). Mechanical and idempotent: re-run it after `git merge origin/master` to move files that
# a concurrent PR added under the old directories and to rewrite their identifiers.
#
#   scripts/rename-java-namespace.sh           apply (git mv + in-place rewrite of tracked text files)
#   scripts/rename-java-namespace.sh --check   exit 1 if anything is still left to rename
#
# Mapping (package / path / Maven):
#   org.ubnfc.java.*  -> org.unlaxer.ubnfc.java.*            (ubnfc-java, the generator)
#   org.ubnfc.lsp.*   -> org.unlaxer.ubnfc.lsp.*             (ubnfc-lsp, LSP/DAP emitters)
#   org.ubnfc.p4.*    -> org.unlaxer.ubnfc.examples.p4.*     (examples/p4-java, p4-vscode)
#   org.ubnfc.ubnf.*  -> org.unlaxer.ubnfc.examples.ubnf.*   (examples/ubnf-vscode)
#   groupId org.ubnfc -> org.unlaxer (artifactIds unchanged)
#
# Deliberately NOT rewritten (historical records of what ran at the time):
#   docs/reports/**, docs/reviews/**, examples/*/evidence/**, this script, and
#   docs/design/03-design-v1.md (its D-076 row records the old -> new mapping).
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo"
mode=apply
[[ "${1:-}" == "--check" ]] && mode=check

self=scripts/rename-java-namespace.sh
excluded() {
  case "$1" in
    docs/reports/*|docs/reviews/*|examples/*/evidence/*|docs/design/03-design-v1.md|"$self") return 0 ;;
  esac
  return 1
}

# old-subpath new-subpath (relative to a Java source root)
moves=(
  "org/ubnfc/java:org/unlaxer/ubnfc/java"
  "org/ubnfc/lsp:org/unlaxer/ubnfc/lsp"
  "org/ubnfc/p4:org/unlaxer/ubnfc/examples/p4"
  "org/ubnfc/ubnf:org/unlaxer/ubnfc/examples/ubnf"
)

pending=0

# 1. Directories: move every tracked file individually so that a target directory that already exists
#    (second run, after a merge) is fine.
while IFS= read -r file; do
  excluded "$file" && continue
  for m in "${moves[@]}"; do
    old=${m%%:*} new=${m#*:}
    case "$file" in
      */"$old"/*)
        dest=${file/\/$old\//\/$new\/}
        pending=1
        if [[ "$mode" == apply ]]; then
          mkdir -p "$(dirname "$dest")"
          git mv -k "$file" "$dest"
        else
          echo "move: $file -> $dest"
        fi
        break ;;
    esac
  done
done < <(git ls-files | grep -E '(^|/)org/ubnfc/(java|lsp|p4|ubnf)/' || true)
if [[ "$mode" == apply ]]; then
  # Remove the now-empty old directories (only directories whose whole subtree has no file).
  while IFS= read -r dir; do
    if [[ -z $(find "$dir" -type f -print -quit) ]]; then rm -r "$dir"; fi
  done < <(find . \( -name .git -o -name target -o -name node_modules \) -prune -o \
             -type d -path '*/org/ubnfc' -print)
fi

# 2. Identifiers in tracked text files. Order matters: the specific sub-packages first, then groupIds.
sed_script='
s#\borg\.ubnfc\.java\b#org.unlaxer.ubnfc.java#g
s#\borg\.ubnfc\.lsp\b#org.unlaxer.ubnfc.lsp#g
s#\borg\.ubnfc\.p4\b#org.unlaxer.ubnfc.examples.p4#g
s#\borg\.ubnfc\.ubnf\b#org.unlaxer.ubnfc.examples.ubnf#g
s#\borg/ubnfc/java\b#org/unlaxer/ubnfc/java#g
s#\borg/ubnfc/lsp\b#org/unlaxer/ubnfc/lsp#g
s#\borg/ubnfc/p4\b#org/unlaxer/ubnfc/examples/p4#g
s#\borg/ubnfc/ubnf\b#org/unlaxer/ubnfc/examples/ubnf#g
s#<groupId>org\.ubnfc</groupId>#<groupId>org.unlaxer</groupId>#g
s#\borg\.ubnfc:ubnfc-#org.unlaxer:ubnfc-#g
'
while IFS= read -r file; do
  excluded "$file" && continue
  [[ -f "$file" ]] || continue
  grep -Iq . "$file" 2>/dev/null || continue   # skip binaries
  if [[ "$mode" == apply ]]; then
    sed -i -E "$sed_script" "$file"
  else
    pending=1; echo "rewrite: $file"
  fi
done < <(git ls-files -z | xargs -0 grep -lIE 'org[./]ubnfc' -- 2>/dev/null || true)

# 3. Whatever is still left (outside the historical records) needs a human look.
left=$(git ls-files -z | xargs -0 grep -nIE 'org[./]ubnfc' -- 2>/dev/null | while IFS= read -r line; do
  excluded "${line%%:*}" || echo "$line"; done || true)
if [[ -n "$left" ]]; then
  echo "remaining org.ubnfc mentions (not covered by the mapping):" >&2
  echo "$left" >&2
  exit 1
fi
if [[ "$mode" == check && "$pending" == 1 ]]; then exit 1; fi
echo "ok: Java namespace is org.unlaxer.ubnfc"
