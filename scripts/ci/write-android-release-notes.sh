#!/usr/bin/env bash
set -euo pipefail

RELEASE_TAG_PATTERN='^v[0-9]+\.[0-9]+(\.[0-9]+)?(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

is_release_tag() {
  local tag="$1"
  [[ "$tag" =~ $RELEASE_TAG_PATTERN ]]
}

resolve_tag_commit() {
  local tag="$1"
  git rev-parse -q --verify "${tag}^{commit}" 2>/dev/null
}

read_sha256_file() {
  local sha_file="$1"
  local sha=""

  [[ -f "$sha_file" ]] || fail "APK_SHA256 is empty and ${sha_file} does not exist."
  read -r sha _ <"$sha_file" || true
  [[ -n "$sha" ]] || fail "Could not read SHA-256 from ${sha_file}."
  printf '%s\n' "$sha"
}

find_previous_release_tag() {
  local current_tag="$1"
  local current_commit="$2"
  local tag
  local tag_commit

  while IFS= read -r tag; do
    [[ -n "$tag" ]] || continue
    [[ "$tag" != "$current_tag" ]] || continue
    is_release_tag "$tag" || continue

    if ! tag_commit="$(resolve_tag_commit "$tag")"; then
      continue
    fi

    [[ "$tag_commit" != "$current_commit" ]] || continue
    printf '%s\n' "$tag"
    return 0
  done < <(git tag --merged "$current_commit" --sort=-creatordate)

  return 1
}

collect_commit_subjects() {
  local range_spec="$1"
  local first_release="$2"
  local -a max_count=()
  local -a subjects=()

  if [[ "$first_release" == "true" ]]; then
    max_count=(--max-count=50)
  fi

  mapfile -t subjects < <(git log --first-parent --format='%s' "${max_count[@]}" "$range_spec")

  if ((${#subjects[@]} == 0)); then
    mapfile -t subjects < <(git log --format='%s' "${max_count[@]}" "$range_spec")
  fi

  if ((${#subjects[@]} == 0)); then
    printf -- '- 本次发布没有检测到新的提交说明。\n'
    return 0
  fi

  printf '%s\n' "${subjects[@]}" | python3 -c 'import sys
for line in sys.stdin.read().splitlines():
    subject = line.strip()
    if subject:
        print(f"- {subject}")
'
}

main() {
  local release_tag="${RELEASE_TAG:-}"
  local apk_asset_name
  local apk_sha256="${APK_SHA256:-}"
  local repository="${GITHUB_REPOSITORY:-rustella/StellarTrail}"
  local run_id="${GITHUB_RUN_ID:-}"
  local current_commit
  local current_short_sha
  local previous_tag=""
  local first_release="false"
  local range_spec
  local download_url
  local notes_file
  local run_url=""
  local commit_bullets

  [[ -n "$release_tag" ]] || fail "RELEASE_TAG is required."
  is_release_tag "$release_tag" || fail "RELEASE_TAG ${release_tag} does not match the Android release tag pattern."

  apk_asset_name="${APK_ASSET_NAME:-StellarTrail-${release_tag}-android-debug.apk}"

  if [[ -z "$apk_sha256" ]]; then
    apk_sha256="$(read_sha256_file "release/${apk_asset_name}.sha256")"
  fi

  current_commit="$(resolve_tag_commit "$release_tag")" || fail "Release tag ${release_tag} does not resolve to a commit."
  current_short_sha="$(git rev-parse --short=12 "$current_commit")"

  if previous_tag="$(find_previous_release_tag "$release_tag" "$current_commit")"; then
    range_spec="${previous_tag}..${release_tag}"
  else
    first_release="true"
    range_spec="$current_commit"
  fi

  commit_bullets="$(collect_commit_subjects "$range_spec" "$first_release")"

  mkdir -p release
  notes_file="release/release-notes-${release_tag}.md"
  download_url="https://github.com/${repository}/releases/download/${release_tag}/${apk_asset_name}"

  if [[ -n "$run_id" ]]; then
    run_url="https://github.com/${repository}/actions/runs/${run_id}"
  fi

  {
    printf '# StellarTrail Android %s\n\n' "$release_tag"
    printf '### 下载\n\n'
    printf -- '- [下载 Android APK](%s)\n' "$download_url"
    printf -- '- SHA-256: `%s`\n\n' "$apk_sha256"
    printf '### 更新内容\n\n'
    printf '%s\n\n' "$commit_bullets"
    printf '### 构建信息\n\n'
    printf -- '- Tag: `%s`\n' "$release_tag"
    printf -- '- Commit: `%s`\n' "$current_short_sha"
    if [[ -n "$previous_tag" ]]; then
      printf -- '- Previous tag: `%s`\n' "$previous_tag"
    else
      printf -- '- Previous tag: 无（首次发布，最多列出 50 条提交）\n'
    fi
    if [[ -n "$run_url" ]]; then
      printf -- '- Actions run: [%s](%s)\n' "$run_id" "$run_url"
    fi
  } >"$notes_file"

  printf '%s\n' "$notes_file"
}

main "$@"
