#!/usr/bin/env bash
# Cross-platform symlink function.

FROM="$1"
TO="$2"

if [[ -z "$TO" ]] ; then
  echo "Call this script as follow: '$0 \$FILE_TO_LINK_TO \$VIRTUAL_LINK_NAME'"
  echo "Order is similar to unix 'ln -s'. It works like 'cp'"
  exit 1
fi

if [[ -n "$WINDIR" ]] ; then
    # Windows needs to be told if it's a directory or not. Infer that.
    # Also: note that we convert `/` to `\`. In this case it's necessary.
    if [[ -d "$FROM" ]]; then
        cmd <<< "mklink /D \"$TO\" \"${FROM//\//\\}\"" > /dev/null
    else
        cmd <<< "mklink \"$TO\" \"${FROM//\//\\}\"" > /dev/null
    fi
else
    ln -s "$FROM" "$TO"
fi
