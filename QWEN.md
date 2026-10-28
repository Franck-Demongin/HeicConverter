# HEIC Converter

HEIC Converter convertit des images HEIC/HEIF en JPEG ou PNG. Le noyau est séparé de l'interface, et l'interface Slint est facultative derrière le feature `gui`.

## Artefacts

- Linux : `heic-converter` et `heic-converter-gui`
- Windows : `heic-converter.exe` et `heic-converter-gui.exe`

## Architecture

- `heic-converter-core` : logique métier et erreurs typées, sans dépendance Slint.
- `heic-converter-cli` : point d'entrée et interface dépendante de `heic-converter-core`.
- `heic-converter-cli/ui/main_window.slint` : vue traitée par `slint-build` dans `build.rs`.
- Le fichier `.slint` est racine du traité, la logique reste côté Rust.
- `slint::set_xdg_app_id` est appelé après création de la fenêtre.
- Notifications toast réutilisables : `docs/toast-integration.md`.
- Référence visuelle GUI : `docs/gui-reference.md`.

## Compilation

Compilation par défaut (ligne de commande uniquement) :

```sh
cargo build -p heic-converter-cli
```

Compilation de l'interface graphique :

```sh
cargo build -p heic-converter-cli --features gui --bin heic-converter-gui
```

Lancement de l'interface graphique :

```sh
cargo run -p heic-converter-cli --features gui --bin heic-converter-gui
```

Compilation complète de la workspace :

```sh
cargo build --workspace --all-features
```

## Installation par Cargo

```sh
cargo install --locked --git https://github.com/Franck-Demongin/heic-converter.git --branch master --features gui
```

Cargo installe `heic-converter` et `heic-converter-gui` sur Linux, `heic-converter.exe` et `heic-converter-gui.exe` sur Windows. Sur cette machine, `~/.cargo/bin` n'est pas forcément dans `PATH`.

## Vérification

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p heic-converter-cli
cargo check -p heic-converter-cli --features gui
```

## Dépendances système pour la carte Linux

Pour Slint 1.18.1 et le backend choisi, winit avec le rendu `femtovg`, installer les paquets vérifiés localement :

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config \
  libfontconfig-dev libfreetype-dev libx11-dev \
  libxkbcommon-dev libwayland-dev libgl-dev
```

## Application Linux

- Identifiant d'application : `io.github.Franck-Demongin.HeicConverter`
- Bureau : `packaging/linux/io.github.Franck-Demongin.HeicConverter.desktop`
- Installation locale : `packaging/linux/install-local.sh` après `cargo install --path crates/cli --features gui`
- Icônes installées sous `io.github.Franck-Demongin.HeicConverter` dans `~/.local/share/icons/hicolor`
- La sélection de l'identifiant est faite après la création du composant, avant l'affichage.

## Arborescence

```text
Cargo.toml
crates/core
crates/cli
packaging/linux/io.github.Franck-Demongin.HeicConverter.desktop
packaging/linux/install-local.sh
packaging/linux/uninstall-local.sh
README.md
QWEN.md
```

## Capture d'interface

Aucune capture n'est actuellement versionnée. Ajoutez les captures disponibles dans les sections correspondantes sans masquer les dépendances ci-dessus.

## Documentation Slint (via slint-docs)

- Les modifications de l'interface graphique sont réalisées avec l'agent `slint-ui` et le serveur MCP `slint-docs`.
- Avant toute modification d'un fichier `.slint`, utiliser le serveur MCP `slint-docs` :
  1. `slint_docs_search` avec un mot-clé précis ;
  2. `slint_reference_describe` avant d'utiliser ou de modifier un widget, élément ou type ;
  3. `slint_docs_get_page` pour les détails de syntaxe ou les exemples ;
  4. `slint_docs_list` si le terme exact est inconnu.
- Serveur : `/home/franck/Dev/MCP/slint_docs_mcp`, lancé avec `uv --directory ... python -m slint_docs_mcp --transport stdio`.
- Documentation Material : une seconde instance MCP `slint-docs-material` utilise la même source, mais `SLINT_DOCS_SOURCE_PATH=ui-libraries/material/docs/src/content/docs`, un index SQLite distinct (`.local/slint-docs-material-index.sqlite3`) et `SLINT_DOCS_PUBLIC_BASE_URL=https://material.slint.dev`. L'agent `slint-ui` a accès à ses tools `mcp__slint-docs-material__*`.
- Utiliser `slint-docs` pour la documentation Slint principale et `slint-docs-material` pour les composants Material ; `https://material.slint.dev` reste le repli si le chemin attendu est absent.
- Version Slint documentée : celle de `Cargo.lock` (1.18.1), donc `SLINT_DOCS_VERSION=1`. Ne pas intégrer d'API absente de cette version.
- Vérifier `page_url` en croisant avec la documentation publiée.
- Conserver les règles Slint existantes et exécuter les tests après modification.

## Règle de publication

Ne jamais pousser, publier ou mettre à jour le dépôt distant (`git push`, tag distant ou release) sans l'accord explicite de l'utilisateur. Avant tout push, demander confirmation à l'utilisateur et attendre sa réponse.

## Licence

GPL-3.0-or-later. La licence de Slint open source impose une licence compatible pour une distribution open source. Les licences commerciales de Slint sont nécessaires pour une distribution propriétaire.