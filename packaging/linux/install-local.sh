#!/usr/bin/env bash
set -euo pipefail

APP_ID="io.github.Franck-Demongin.HeicConverter"
APP_NAME="HEIC Converter"
ICON_NAME="$APP_ID"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ASSETS_DIR="$PROJECT_ROOT/assets/icons"

if [ ! -d "$ASSETS_DIR" ]; then
  echo "Répertoire d'icônes introuvable : $ASSETS_DIR" >&2
  exit 1
fi

GUI_BINARY_PATH=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary)
      if [[ $# -lt 2 ]]; then
        echo "Option --binary sans chemin." >&2
        exit 1
      fi
      GUI_BINARY_PATH="$2"
      shift 2
      ;;
    --binary=*)
      GUI_BINARY_PATH="${1#*=}"
      shift
      ;;
    --help|-h)
      cat <<'EOF'
Usage: install-local.sh [--binary CHEMIN]

Sans option, le script recherche heic-converter-gui dans le PATH,
puis dans ~/.cargo/bin.

Options :
  --binary CHEMIN   Chemin explicite vers le binaire heic-converter-gui.
  -h, --help       Affiche cette aide.
EOF
      exit 0
      ;;
    --)
      shift
      break
      ;;
    *)
      echo "Option inconnue : $1" >&2
      exit 1
      ;;
  esac
done

if [[ -n "$GUI_BINARY_PATH" ]]; then
  if [[ ! -x "$GUI_BINARY_PATH" ]]; then
    echo "Binaire introuvable ou non executable : $GUI_BINARY_PATH" >&2
    exit 1
  fi

  # Installe le binaire demande dans le repertoire utilisateur habituel,
  # puis integre ce chemin stable au bureau Linux.
  INSTALL_BIN_DIR="$HOME/.local/bin"
  INSTALL_BIN="$INSTALL_BIN_DIR/heic-converter-gui"
  mkdir -p "$INSTALL_BIN_DIR"

  SOURCE_BIN="$GUI_BINARY_PATH"
  if command -v realpath >/dev/null 2>&1; then
    SOURCE_BIN="$(realpath "$GUI_BINARY_PATH")"
  fi

  if [[ "$SOURCE_BIN" != "$INSTALL_BIN" ]]; then
    install -m 0755 "$GUI_BINARY_PATH" "$INSTALL_BIN"
  fi
  TRIMBOX_BIN="$INSTALL_BIN"
else
  if command -v heic-converter-gui >/dev/null 2>&1; then
    TRIMBOX_BIN="$(command -v heic-converter-gui)"
  elif [[ -x "$HOME/.cargo/bin/heic-converter-gui" ]]; then
    TRIMBOX_BIN="$HOME/.cargo/bin/heic-converter-gui"
    mkdir -p "$HOME/.local/bin"
    ln -sfn "$TRIMBOX_BIN" "$HOME/.local/bin/heic-converter-gui"
  else
    echo "Binaire heic-converter-gui introuvable. Installez-le d'abord avec cargo install --features gui ou utilisez --binary." >&2
    exit 1
  fi
fi

if [[ "$TRIMBOX_BIN" != /* ]] && command -v realpath >/dev/null 2>&1; then
  TRIMBOX_BIN="$(realpath "$TRIMBOX_BIN")"
elif [[ "$TRIMBOX_BIN" != /* ]]; then
  TRIMBOX_BIN="$PWD/$TRIMBOX_BIN"
fi

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS_DIR="$DATA_HOME/applications"
ICONS_ROOT="$DATA_HOME/icons/hicolor"

for size in 16 32 48 64 128 256 512; do
  target_dir="$ICONS_ROOT/${size}x${size}/apps"
  mkdir -p "$target_dir"
  install -m 0644 "$ASSETS_DIR/heic-converter-${size}.png" "$target_dir/${ICON_NAME}.png"
done

mkdir -p "$ICONS_ROOT/scalable/apps"
install -m 0644 "$ASSETS_DIR/heic-converter.svg" "$ICONS_ROOT/scalable/apps/${ICON_NAME}.svg"

mkdir -p "$APPS_DIR"
installed_desktop="$APPS_DIR/${APP_ID}.desktop"
template="$PROJECT_ROOT/packaging/linux/${APP_ID}.desktop"

sed \
  -e "s|^TryExec=.*|TryExec=${TRIMBOX_BIN}|" \
  -e "s|^Exec=.*|Exec=${TRIMBOX_BIN}|" \
  "$template" > "$installed_desktop"
chmod 0644 "$installed_desktop"

# Remove stale files left by the previous "heic-converter" icon/desktop naming.
# They are not used anymore and can keep GNOME associated with the wrong icon.
rm -f "$APPS_DIR/heic-converter.desktop"
for size in 16 32 48 64 128 256 512; do
  rm -f "$ICONS_ROOT/${size}x${size}/apps/heic-converter.png"
done
rm -f "$ICONS_ROOT/scalable/apps/heic-converter.svg"

if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate "$installed_desktop"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$ICONS_ROOT" >/dev/null 2>&1 || true
fi

echo "Installation terminée."
echo "Identifiant : $APP_ID"
echo "Bureau : $installed_desktop"
echo "Icônes : $ICONS_ROOT"
