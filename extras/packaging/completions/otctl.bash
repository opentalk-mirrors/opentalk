# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Bash completion for otctl
_otctl() {
  local cur prev words cword
  _init_completion 2>/dev/null || {
    # Minimal fallback ohne bash-completion Framework
    cur="${COMP_WORDS[COMP_CWORD]}"
  }
  local global_opts="--help -h --version -V --verbose -v --config --dry-run"
  local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/otctl"
  local cache_file="$cache_dir/help.txt"
  local cache_ttl=60
  if [[ ! -s "$cache_file" || $(( $(date +%s) - $(stat -c %Y "$cache_file" 2>/dev/null || echo 0) )) -gt $cache_ttl ]]; then
    mkdir -p "$cache_dir"
    (otctl --help 2>/dev/null || otctl help 2>/dev/null || true) >"$cache_file" 2>/dev/null || true
  fi
  local subs
  subs=$(grep -E '^[[:space:]]+[a-z0-9_-]+[[:space:]]+-' "$cache_file" | awk '{print $1}' | sort -u)
  local cword=${COMP_CWORD}
  local cur="${COMP_WORDS[COMP_CWORD]}"
  if [[ $cword -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "$subs $global_opts" -- "$cur") )
    return
  fi
  local sub
  for w in "${COMP_WORDS[@]:1}"; do
    if grep -q -w "$w" <<<"$subs"; then sub="$w"; break; fi
  done
  case "$sub" in
    deploy)   COMPREPLY=( $(compgen -W "--force --limit --tags --skip-tags --extra-vars -e --check" -- "$cur") );;
    upgrade)  COMPREPLY=( $(compgen -W "--force --backup --tags --check" -- "$cur") );;
    status)   COMPREPLY=( $(compgen -W "--json --services --since --follow" -- "$cur") );;
    logs)     COMPREPLY=( $(compgen -W "--since --follow --service --tail" -- "$cur") );;
    config)   COMPREPLY=( $(compgen -W "diff validate show path" -- "$cur") );;
    migrate)  COMPREPLY=( $(compgen -W "minio-to-garage garage-to-minio --no-deploy" -- "$cur") );;
    *)        COMPREPLY=( $(compgen -W "$global_opts" -- "$cur") );;
  esac
}
complete -F _otctl otctl