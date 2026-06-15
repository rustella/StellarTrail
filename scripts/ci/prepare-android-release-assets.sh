#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

find_single_file() {
  local search_dir="$1"
  local pattern="$2"
  local description="$3"
  local -a files=()

  while IFS= read -r file; do
    files+=("$file")
  done < <(find "$search_dir" -maxdepth 1 -type f -name "$pattern" | sort)

  if ((${#files[@]} == 0)); then
    fail "No ${description} found under ${search_dir}."
  fi

  if ((${#files[@]} > 1)); then
    printf 'Multiple %s files found; refusing to guess:\n' "$description" >&2
    printf '  %s\n' "${files[@]}" >&2
    exit 1
  fi

  printf '%s\n' "${files[0]}"
}

require_file() {
  local path="$1"
  local description="$2"

  [[ -s "$path" ]] || fail "${description} is missing or empty: ${path}"
}

ensure_native_symbols_archive() {
  local native_symbols="$1"
  local native_libs_dir="apps/android/build/intermediates/merged_native_libs/release/mergeReleaseNativeLibs/out/lib"
  local native_symbols_abs
  local -a native_libs=()

  if [[ -s "$native_symbols" ]]; then
    return 0
  fi

  [[ -d "$native_libs_dir" ]] || fail "Native libraries directory is missing: ${native_libs_dir}"
  while IFS= read -r native_lib; do
    native_libs+=("$native_lib")
  done < <(find "$native_libs_dir" -type f -name '*.so' | sort)
  if ((${#native_libs[@]} == 0)); then
    fail "No native libraries found under ${native_libs_dir}."
  fi

  mkdir -p "$(dirname "$native_symbols")"
  native_symbols_abs="$(pwd)/${native_symbols}"
  (
    cd "$native_libs_dir"
    jar cf "$native_symbols_abs" .
  )
}

verify_aab_metadata() {
  local aab_src="$1"
  local listing

  listing="$(jar tf "$aab_src")"
  printf '%s\n' "$listing" |
    grep -F -q 'BUNDLE-METADATA/com.android.tools.build.obfuscation/' ||
    fail "Release AAB does not contain R8 deobfuscation metadata."
}

main() {
  local asset_prefix="${1:-}"
  local apk_src
  local aab_src
  local mapping_file="apps/android/build/outputs/mapping/release/mapping.txt"
  local native_symbols="apps/android/build/outputs/native-debug-symbols/release/native-debug-symbols.zip"
  local apk_name
  local aab_name
  local native_symbols_name
  local apk_sha256
  local aab_sha256
  local native_symbols_sha256
  local release_dir="${RELEASE_DIR:-release}"

  [[ -n "$asset_prefix" ]] || fail "Usage: $0 <asset-prefix>"
  [[ -n "${ANDROID_HOME:-}" ]] || fail "ANDROID_HOME is required."

  require_file "$mapping_file" "R8 mapping file"
  ensure_native_symbols_archive "$native_symbols"
  require_file "$native_symbols" "Native debug symbols archive"

  apk_src="$(find_single_file "apps/android/build/outputs/apk/release" '*.apk' 'release APK')"
  aab_src="$(find_single_file "apps/android/build/outputs/bundle/release" '*.aab' 'release AAB')"

  "${ANDROID_HOME}/build-tools/36.0.0/apksigner" verify --verbose "$apk_src"
  jarsigner -verify "$aab_src" >/dev/null
  verify_aab_metadata "$aab_src"

  mkdir -p "$release_dir"
  apk_name="${asset_prefix}.apk"
  aab_name="${asset_prefix}.aab"
  native_symbols_name="${asset_prefix}-native-debug-symbols.zip"
  cp "$apk_src" "${release_dir}/${apk_name}"
  cp "$aab_src" "${release_dir}/${aab_name}"
  cp "$native_symbols" "${release_dir}/${native_symbols_name}"

  (
    cd "$release_dir"
    sha256sum "$apk_name" >"${apk_name}.sha256"
    sha256sum "$aab_name" >"${aab_name}.sha256"
    sha256sum "$native_symbols_name" >"${native_symbols_name}.sha256"
  )
  read -r apk_sha256 _ <"${release_dir}/${apk_name}.sha256"
  read -r aab_sha256 _ <"${release_dir}/${aab_name}.sha256"
  read -r native_symbols_sha256 _ <"${release_dir}/${native_symbols_name}.sha256"

  if [[ -n "${GITHUB_ENV:-}" ]]; then
    {
      printf 'APK_ASSET_NAME=%s\n' "$apk_name"
      printf 'AAB_ASSET_NAME=%s\n' "$aab_name"
      printf 'NATIVE_SYMBOLS_ASSET_NAME=%s\n' "$native_symbols_name"
      printf 'APK_SHA256=%s\n' "$apk_sha256"
      printf 'AAB_SHA256=%s\n' "$aab_sha256"
      printf 'NATIVE_SYMBOLS_SHA256=%s\n' "$native_symbols_sha256"
      printf 'ANDROID_ARTIFACT_NAME=%s\n' "$asset_prefix"
    } >>"$GITHUB_ENV"
  fi

  printf 'Prepared Android release assets:\n'
  printf '  %s\n' "${release_dir}/${apk_name}"
  printf '  %s\n' "${release_dir}/${aab_name}"
  printf '  %s\n' "${release_dir}/${native_symbols_name}"
}

main "$@"
