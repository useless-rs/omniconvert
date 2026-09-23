#!/usr/bin/env bash
set -u
tools=(ffmpeg magick convert pandoc libreoffice 7z gs inkscape ebook-convert blender fontforge qpdf)
missing=0
for t in "${tools[@]}"; do
  if command -v "$t" >/dev/null 2>&1; then echo "OK   $t ($(command -v "$t"))";
  else echo "MISS $t"; missing=$((missing+1)); fi
done
echo "---"
if [ "$missing" -eq 0 ]; then echo "All tools present."; else echo "$missing tool(s) missing. See scripts/install-deps.sh"; fi
