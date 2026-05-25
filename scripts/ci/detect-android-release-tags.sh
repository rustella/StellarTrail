#!/usr/bin/env bash
set -euo pipefail

RELEASE_TAG_PATTERN='^v[0-9]+\.[0-9]+(\.[0-9]+)?(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'

selected_tags=()
summary_lines=()

log() {
  printf '%s\n' "$*" >&2
}

add_summary() {
  summary_lines+=("$*")
}

is_release_tag() {
  local tag="$1"
  [[ "$tag" =~ $RELEASE_TAG_PATTERN ]]
}

is_zero_sha() {
  local sha="$1"
  [[ -n "$sha" && "$sha" =~ ^0+$ ]]
}

resolve_tag_commit() {
  local tag="$1"
  git rev-parse -q --verify "${tag}^{commit}" 2>/dev/null
}

is_ancestor() {
  local ancestor="$1"
  local descendant="$2"
  git merge-base --is-ancestor "$ancestor" "$descendant" >/dev/null 2>&1
}

fetch_release_refs() {
  git fetch --force --tags origin '+refs/tags/*:refs/tags/*' '+refs/heads/main:refs/remotes/origin/main'
}

release_exists() {
  local tag="$1"

  if [[ "${SKIP_GH_RELEASE_CHECK:-0}" == "1" ]]; then
    return 1
  fi

  if ! command -v gh >/dev/null 2>&1; then
    log "gh was not found; assuming release ${tag} does not exist. Set SKIP_GH_RELEASE_CHECK=1 for local tests."
    return 1
  fi

  GH_PROMPT_DISABLED=1 gh release view "$tag" >/dev/null 2>&1
}

append_unique_tag() {
  local tag="$1"
  local commit="$2"
  local reason="$3"
  local existing

  for existing in "${selected_tags[@]}"; do
    if [[ "$existing" == "$tag" ]]; then
      return 0
    fi
  done

  selected_tags+=("$tag")
  add_summary "- Selected \`${tag}\` (${commit:0:12})${reason:+: ${reason}}."
}

consider_release_tag() {
  local tag="$1"
  local require_main_ancestor="$2"
  local reason="$3"
  local commit

  if ! is_release_tag "$tag"; then
    add_summary "- Skipped \`${tag}\`: tag name does not match the Android release pattern."
    return 0
  fi

  if ! commit="$(resolve_tag_commit "$tag")"; then
    add_summary "- Skipped \`${tag}\`: tag does not resolve to a commit."
    return 0
  fi

  if [[ "$require_main_ancestor" == "true" ]] && ! is_ancestor "$commit" "origin/main"; then
    add_summary "- Skipped \`${tag}\`: tag target ${commit:0:12} is not reachable from \`origin/main\`."
    return 0
  fi

  if release_exists "$tag"; then
    add_summary "- Skipped \`${tag}\`: GitHub Release already exists."
    return 0
  fi

  append_unique_tag "$tag" "$commit" "$reason"
}

read_before_from_event_payload() {
  local event_path="${GITHUB_EVENT_PATH:-}"

  if [[ -z "$event_path" || ! -f "$event_path" ]]; then
    return 0
  fi

  python3 - "$event_path" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("before", ""))
PY
}

detect_main_push_tags() {
  local new_sha="${GITHUB_SHA:-}"
  local before_sha="${GITHUB_EVENT_BEFORE:-}"
  local tag
  local commit

  if [[ -z "$new_sha" ]]; then
    printf 'GITHUB_SHA is required for push events on main.\n' >&2
    exit 1
  fi

  if ! git cat-file -e "${new_sha}^{commit}" 2>/dev/null; then
    printf 'GITHUB_SHA %s does not resolve to a commit.\n' "$new_sha" >&2
    exit 1
  fi

  if [[ -z "$before_sha" ]]; then
    before_sha="$(read_before_from_event_payload)"
  fi

  if is_zero_sha "$before_sha"; then
    add_summary "- Previous main SHA is all zeroes; treating this as the first push."
    before_sha=""
  elif [[ -n "$before_sha" ]] && ! git cat-file -e "${before_sha}^{commit}" 2>/dev/null; then
    add_summary "- Previous main SHA \`${before_sha}\` was not found locally; treating reachable release tags as new."
    before_sha=""
  elif [[ -z "$before_sha" ]]; then
    add_summary "- Previous main SHA is empty; treating reachable release tags as new."
  fi

  while IFS= read -r tag; do
    [[ -n "$tag" ]] || continue
    is_release_tag "$tag" || continue

    if ! commit="$(resolve_tag_commit "$tag")"; then
      add_summary "- Skipped \`${tag}\`: tag does not resolve to a commit."
      continue
    fi

    if [[ -n "$before_sha" ]] && is_ancestor "$commit" "$before_sha"; then
      continue
    fi

    consider_release_tag "$tag" "false" "newly reachable from main"
  done < <(git tag --merged "$new_sha" --sort=version:refname)
}

detect_single_tag() {
  local tag="$1"
  local source="$2"

  if [[ -z "$tag" ]]; then
    add_summary "- No tag was provided for ${source}; no Android Release will be created."
    return 0
  fi

  consider_release_tag "$tag" "true" "from ${source}"
}

json_array() {
  if ((${#selected_tags[@]} == 0)); then
    printf '[]\n'
    return 0
  fi

  printf '%s\0' "${selected_tags[@]}" | python3 -c 'import json, sys; print(json.dumps([item.decode("utf-8") for item in sys.stdin.buffer.read().split(b"\0") if item], ensure_ascii=False))'
}

emit_outputs() {
  local json="$1"
  local has_tags="$2"

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
      printf 'tags_json=%s\n' "$json"
      printf 'has_tags=%s\n' "$has_tags"
    } >>"$GITHUB_OUTPUT"
  else
    printf 'tags_json=%s\n' "$json"
    printf 'has_tags=%s\n' "$has_tags"
  fi
}

write_step_summary() {
  local json="$1"
  local has_tags="$2"
  local event="${GITHUB_EVENT_NAME:-unknown}"
  local ref="${GITHUB_REF:-unknown}"
  local line

  if [[ -z "${GITHUB_STEP_SUMMARY:-}" ]]; then
    return 0
  fi

  {
    printf '### Android Release tag detection\n\n'
    printf -- '- Event: `%s`\n' "$event"
    printf -- '- Ref: `%s`\n' "$ref"
    printf -- '- Release check: `%s`\n' "$(if [[ "${SKIP_GH_RELEASE_CHECK:-0}" == "1" ]]; then printf 'skipped'; else printf 'enabled'; fi)"
    printf -- '- Result: `%s` (%s)\n\n' "$has_tags" "$json"

    if ((${#summary_lines[@]} == 0)); then
      printf 'No matching Android release tags were found.\n'
    else
      for line in "${summary_lines[@]}"; do
        printf '%s\n' "$line"
      done
    fi
  } >>"$GITHUB_STEP_SUMMARY"
}

main() {
  local event="${GITHUB_EVENT_NAME:-}"
  local ref="${GITHUB_REF:-}"
  local input_tag="${INPUT_TAG:-}"
  local tags_json
  local has_tags="false"

  fetch_release_refs

  case "$event:$ref" in
    push:refs/heads/main)
      detect_main_push_tags
      ;;
    push:refs/tags/*)
      detect_single_tag "${ref#refs/tags/}" "tag push"
      ;;
    workflow_dispatch:*)
      detect_single_tag "$input_tag" "workflow_dispatch"
      ;;
    *)
      add_summary "- Unsupported event/ref combination \`${event:-unknown}\` / \`${ref:-unknown}\`; no Android Release will be created."
      ;;
  esac

  tags_json="$(json_array)"
  if ((${#selected_tags[@]} > 0)); then
    has_tags="true"
  fi

  emit_outputs "$tags_json" "$has_tags"
  write_step_summary "$tags_json" "$has_tags"
}

main "$@"
