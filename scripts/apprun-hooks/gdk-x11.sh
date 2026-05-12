#!/usr/bin/env bash
set -e

_hook_dir=$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)
export APPDIR="${APPDIR:-$(dirname "$_hook_dir")}"
unset _hook_dir
cd "$APPDIR" || true

export GDK_BACKEND=x11
export WINIT_UNIX_BACKEND=x11

export GTK_MODULES=""



if [ -z "${XAUTHORITY:-}" ]; then
  if [ -f "${HOME}/.Xauthority" ]; then
    export XAUTHORITY="${HOME}/.Xauthority"
  elif [ -n "${XDG_RUNTIME_DIR:-}" ]; then
    for cand in "${XDG_RUNTIME_DIR}"/.mutter-Xwaylandauth.* "${XDG_RUNTIME_DIR}"/xauth_*; do
      if [ -f "$cand" ]; then
        export XAUTHORITY="$cand"
        break
      fi
    done
  fi
fi
