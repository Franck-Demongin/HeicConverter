# HEIC Converter

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

HEIC Converter est un outil Rust de ligne de commande avec une interface Slint optionnelle. Il convertit des images HEIC/HEIF en JPEG ou PNG et conserve la orientation contenue dans le fichier HEIC.

Le noyau métier reste séparé de l'interface : la GUI est activée par la feature `gui`.

## Fonctionnalités

- Conversion de fichiers HEIC/HEIF en JPEG ou PNG.
- Application des transformations contenues (`irot`, `imir`, `clap`).
- Conservation de la resolution et des dimensions source.
- Traitement par lots via la ligne de commande.
- Extraction simple d'un rapport de conversion dans l'interface graphique.
- toasts et interface Slint fondes sur la palette systeme.

## Installation

### Depuis GitHub

```sh
cargo install --locked --git https://github.com/Franck-Demongin/heic-converter.git --branch main --features gui
```

Cette commande installe `heic-converter` et `heic-converter-gui` sur Linux, `heic-converter.exe` et `heic-converter-gui.exe` sur Windows. Sur certaines machines, `~/.cargo/bin` n'est pas dans `PATH`.

### Depuis une copie locale

```sh
cargo install --path crates/cli --features gui
```

### Dépendances de compilation sur Linux

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config \
  libfontconfig-dev libfreetype-dev libx11-dev \
  libxkbcommon-dev libwayland-dev libgl-dev
```

### Intégration Linux (bureau et icône)

Depuis une copie du dépôt :

```sh
packaging/linux/install-local.sh
```

## Utilisation CLI

```sh
heic-converter convert input.heic --output ./sortie --format jpeg --quality 90
heic-converter convert ./images --output ./sortie --format png
```

## Compilation

```sh
cargo build -p heic-converter-cli
cargo build -p heic-converter-cli --features gui --bin heic-converter-gui
cargo build --workspace --all-features
```

## Vérification

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Licence

GPL-3.0-or-later. La licence de Slint open source impose une licence compatible pour une distribution open source. Les licences commerciales de Slint sont nécessaires pour une distribution propriétaire.