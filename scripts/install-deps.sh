#!/usr/bin/env bash
set -u
echo "== macOS (brew) =="
echo "brew install ffmpeg imagemagick pandoc libreoffice p7zip ghostscript inkscape calibre blender fontforge qpdf"
echo "== Ubuntu/Debian =="
echo "sudo apt install ffmpeg imagemagick pandoc libreoffice p7zip-full ghostscript inkscape calibre blender fontforge qpdf"
echo "== Windows (choco) =="
echo "choco install ffmpeg imagemagick pandoc libreoffice 7zip ghostscript inkscape calibre blender fontforge qpdf -y"
echo "Run scripts/check-deps.sh after installing."
