#![cfg(feature = "gui")]

mod toast;
use toast::{ToastLevel, Toaster};

use anyhow::{Context, Result};
use heic_converter_core::{ConversionOptions, OutputFormat, convert_file};
use slint::ComponentHandle;
use std::path::{Path, PathBuf};
use std::process::Command;

slint::include_modules!();

pub fn run() -> Result<()> {
    let ui = MainWindow::new()
        .map_err(|error| anyhow::anyhow!("creation de la fenetre impossible : {error}"))?;

    ui.set_window_title(format!("HEIC Converter - v{}", env!("CARGO_PKG_VERSION")).into());

    let _ = slint::set_xdg_app_id("io.github.Franck-Demongin.HeicConverter");

    let weak = ui.as_weak();

    let toaster = Toaster::install(&ui);

    let format_weak = weak.clone();
    ui.on_format_changed(move || {
        if let Some(ui) = format_weak.upgrade() {
            update_output_extension(&ui);
        }
    });

    let browse_source_weak = weak.clone();
    let browse_toaster = toaster.clone();
    ui.on_browse_source(move || {
        if let Some(ui) = browse_source_weak.upgrade() {
            let dialog = rfd::FileDialog::new()
                .add_filter("HEIC/HEIF", &["heic", "heif"])
                .set_title("Ouvrir un fichier HEIC/HEIF");
            if let Some(path) = dialog.pick_file() {
                let source = path.display().to_string();
                let output = default_output_path(&path, ui.get_output_format().as_str());
                ui.set_source_path(source.clone().into());
                ui.set_output_path(output.display().to_string().into());
                ui.set_output_available(false);
                ui.set_report_available(false);
                set_status(&ui, &browse_toaster, "Source chargee.".to_owned(), 0);
            }
        }
    });

    let browse_output_weak = weak.clone();
    ui.on_browse_output(move || {
        if let Some(ui) = browse_output_weak.upgrade() {
            let dialog = rfd::FileDialog::new()
                .add_filter("Image", &["jpg", "jpeg", "png"])
                .set_title("Enregistrer l'image generee");
            if let Some(path) = dialog.save_file() {
                let output = normalized_output_path(path, ui.get_output_format().as_str());
                ui.set_output_path(output.display().to_string().into());
            }
        }
    });

    let convert_weak = weak.clone();
    let convert_toaster = toaster.clone();
    ui.on_convert(move || {
        if let Some(ui) = convert_weak.upgrade() {
            convert_gui(&ui, &convert_toaster);
        }
    });

    let folder_weak = weak.clone();
    let folder_toaster = toaster.clone();
    ui.on_open_output_folder(move || {
        if let Some(ui) = folder_weak.upgrade() {
            open_output_folder_gui(&ui, &folder_toaster);
        }
    });

    let batch_source_weak = weak.clone();
    ui.on_browse_batch_source(move || {
        if let Some(ui) = batch_source_weak.upgrade()
            && let Some(path) = rfd::FileDialog::new()
                .set_title("Selectionner le dossier source du lot")
                .pick_folder()
        {
            ui.set_batch_source_path(path.display().to_string().into());
        }
    });

    let batch_output_weak = weak.clone();
    ui.on_browse_batch_output(move || {
        if let Some(ui) = batch_output_weak.upgrade()
            && let Some(path) = rfd::FileDialog::new()
                .set_title("Selectionner le dossier de sortie du lot")
                .pick_folder()
        {
            ui.set_batch_output_path(path.display().to_string().into());
        }
    });

    let file_weak = weak.clone();
    let file_toaster = toaster.clone();
    ui.on_open_output_file(move || {
        if let Some(ui) = file_weak.upgrade() {
            open_output_file_gui(&ui, &file_toaster);
        }
    });

    ui.run()
        .map_err(|error| anyhow::anyhow!("execution de l'interface impossible : {error}"))?;
    Ok(())
}

struct SingleOutcome {
    success: bool,
    output_path: String,
    report: String,
    status: String,
}

fn convert_gui(ui: &MainWindow, toaster: &Toaster) {
    if ui.get_batch_mode() {
        convert_batch_gui(ui, toaster);
        return;
    }

    ui.set_output_available(false);
    ui.set_report_available(false);

    let input = ui.get_source_path().as_str().trim().to_owned();
    if input.is_empty() {
        set_status(
            ui,
            toaster,
            "Choisir d'abord un fichier source.".to_owned(),
            2,
        );
        return;
    }
    let input_path = PathBuf::from(&input);

    let format_str = ui.get_output_format().as_str().trim().to_lowercase();
    let quality: u8 = ui
        .get_quality()
        .as_str()
        .trim()
        .parse()
        .unwrap_or(90)
        .clamp(1, 100);
    let format = match format_str.as_str() {
        "png" => OutputFormat::Png,
        _ => OutputFormat::Jpeg { quality },
    };

    let output = {
        let current = ui.get_output_path();
        let current = current.as_str().trim().to_owned();
        if current.is_empty() {
            default_output_path(&input_path, ui.get_output_format().as_str())
        } else {
            PathBuf::from(current)
        }
    };
    let output = normalized_output_path(output, &format_str);
    ui.set_output_path(output.display().to_string().into());

    if same_path(&input_path, &output) {
        set_status(
            ui,
            toaster,
            "Le fichier de sortie doit differer du fichier source.".to_owned(),
            2,
        );
        return;
    }

    let progress_name = input_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("fichier source")
        .to_owned();
    ui.set_progress_label(format!("Fichier {} en cours de traitement", progress_name).into());
    ui.set_converting(true);

    let ui_weak = ui.as_weak();
    let toaster = toaster.clone();
    let output_text = output.display().to_string();
    std::thread::spawn(move || {
        let options = ConversionOptions {
            format,
            apply_transforms: true,
            max_pixels: 1 << 28,
        };

        let outcome = match convert_file(&input_path, &output, &options) {
            Ok(result) => SingleOutcome {
                success: true,
                output_path: output.display().to_string(),
                report: format!(
                    "HEIC Converter - resultat\nFichier source : {}\nFichier genere : {}\nDimensions : {} x {}\nTaille : {} octets\nAlpha : {}\n16 bits : {}",
                    result.input.display(),
                    result.output.display(),
                    result.width,
                    result.height,
                    result.output_bytes,
                    result.has_alpha,
                    result.sixteen_bit
                ),
                status: format!("Conversion terminee ({}x{}).", result.width, result.height),
            },
            Err(error) => SingleOutcome {
                success: false,
                output_path: output_text.clone(),
                report: String::new(),
                status: format!("Conversion impossible : {error}"),
            },
        };

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                if outcome.success {
                    ui.set_output_path(outcome.output_path.into());
                    ui.set_output_available(true);
                    ui.set_report_title("Resultat".into());
                    ui.set_report(outcome.report.into());
                    ui.set_report_available(true);
                    toaster.show(&outcome.status, ToastLevel::Success);
                } else {
                    toaster.show(&outcome.status, ToastLevel::Error);
                }
                ui.set_progress_label("".into());
                ui.set_converting(false);
            }
        });
    });
}

fn convert_batch_gui(ui: &MainWindow, toaster: &Toaster) {
    ui.set_output_available(false);
    ui.set_report_available(false);

    let source = ui.get_batch_source_path().as_str().trim().to_owned();
    let output = ui.get_batch_output_path().as_str().trim().to_owned();
    if source.is_empty() || output.is_empty() {
        set_status(
            ui,
            toaster,
            "Choisir d'abord les dossiers source et sortie.".to_owned(),
            2,
        );
        return;
    }
    let source_dir = PathBuf::from(source);
    let output_dir = PathBuf::from(output);
    if !source_dir.is_dir() {
        set_status(ui, toaster, "Dossier source introuvable.".to_owned(), 2);
        return;
    }
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        set_status(
            ui,
            toaster,
            format!("Creation du dossier de sortie impossible : {error}"),
            2,
        );
        return;
    }
    if output_dir.is_file() {
        set_status(
            ui,
            toaster,
            "Le chemin de sortie doit etre un dossier.".to_owned(),
            2,
        );
        return;
    }

    let format_str = ui.get_output_format().as_str().trim().to_lowercase();
    let quality: u8 = ui
        .get_quality()
        .as_str()
        .trim()
        .parse()
        .unwrap_or(90)
        .clamp(1, 100);
    let format = match format_str.as_str() {
        "png" => OutputFormat::Png,
        _ => OutputFormat::Jpeg { quality },
    };
    if let OutputFormat::Jpeg { quality } = format
        && !(1..=100).contains(&quality)
    {
        set_status(ui, toaster, "Qualite JPEG invalide.".to_owned(), 2);
        return;
    }

    let extension = match format {
        OutputFormat::Jpeg { .. } => "jpg",
        OutputFormat::Png => "png",
    };

    let mut files = Vec::new();
    match std::fs::read_dir(&source_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_heic_file(&path) {
                    files.push(path);
                }
            }
        }
        Err(error) => {
            set_status(ui, toaster, format!("Lecture impossible : {error}"), 2);
            return;
        }
    }
    files.sort();
    if files.is_empty() {
        set_status(
            ui,
            toaster,
            "Aucun fichier HEIC/HEIF trouve dans le dossier source.".to_owned(),
            2,
        );
        return;
    }

    let naming = ui.get_batch_naming().as_str().trim().to_owned();
    let generic = naming.to_lowercase().contains("generique");
    let base_name = {
        let raw = ui.get_batch_base_name().as_str().trim().to_owned();
        let candidate = if raw.is_empty() {
            "image".to_owned()
        } else {
            raw
        };
        std::path::Path::new(&candidate)
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or("image")
            .to_owned()
    };

    let options = ConversionOptions {
        format,
        apply_transforms: true,
        max_pixels: 1 << 28,
    };

    ui.set_batch_progress(0.0);
    ui.set_converting(true);

    let ui_weak = ui.as_weak();
    let toaster = toaster.clone();
    let file_count = files.len();
    let total_files = file_count.max(1);
    let output_dir_text = output_dir.display().to_string();
    std::thread::spawn(move || {
        let mut successes = 0usize;
        let mut errors = 0usize;
        let mut lines = vec![
            "HEIC Converter - traitement par lot".to_owned(),
            format!("Dossier source : {}", source_dir.display()),
            format!("Dossier de sortie : {}", output_dir.display()),
            format!("Fichiers trouves : {}", file_count),
            String::new(),
        ];

        for (index, input) in files.iter().enumerate() {
            let input_name = input
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("fichier source")
                .to_owned();
            let label_text = format!("Fichier {} - {}/{}", input_name, index + 1, file_count);
            let label_weak = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = label_weak.upgrade() {
                    ui.set_progress_label(label_text.into());
                }
            });

            let output_name = if generic {
                format!("{}_{:03}.{}", base_name, index + 1, extension)
            } else {
                let stem = input
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .filter(|value| !value.is_empty())
                    .unwrap_or("sortie");
                format!("{}.{}", stem, extension)
            };
            let output_path = output_dir.join(output_name);

            match convert_file(input, &output_path, &options) {
                Ok(result) => {
                    successes += 1;
                    lines.push(format!(
                        "OK {} -> {} ({}x{})",
                        input.display(),
                        output_path.display(),
                        result.width,
                        result.height
                    ));
                }
                Err(error) => {
                    errors += 1;
                    lines.push(format!("ERR {} : {error}", input.display()));
                }
            }

            let progress = (index + 1) as f32 / total_files as f32;
            let progress_weak = ui_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = progress_weak.upgrade() {
                    ui.set_batch_progress(progress);
                }
            });
        }

        lines.push(String::new());
        lines.push(format!("Reussies : {successes}"));
        lines.push(format!("Echecs : {errors}"));

        let report = lines.join("\n");
        let output_available = successes > 0;
        let level_int = if errors > 0 { 2 } else { 0 };
        let status = format!("Lot termine : {successes} reussie(s), {errors} echec(s).");
        let report_title = "Resultat du lot".to_owned();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_output_path(output_dir_text.into());
                if output_available {
                    ui.set_output_available(true);
                }
                ui.set_report_title(report_title.into());
                ui.set_report(report.into());
                ui.set_report_available(true);
                let level = match level_int {
                    2 => ToastLevel::Error,
                    _ => ToastLevel::Success,
                };
                ui.set_batch_progress(1.0);
                ui.set_progress_label("".into());
                toaster.show(&status, level);
                ui.set_converting(false);
            }
        });
    });
}

fn is_heic_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_lowercase();
            ext == "heic" || ext == "heif"
        })
        .unwrap_or(false)
}

fn set_status(
    _ui: &MainWindow,
    toaster: &Toaster,
    status: impl Into<slint::SharedString>,
    level: i32,
) {
    let level = match level {
        1 => ToastLevel::Warning,
        2 => ToastLevel::Error,
        _ => ToastLevel::Success,
    };
    toaster.show(status.into().as_str().to_owned(), level);
}

fn default_output_path(input: &Path, format: &str) -> PathBuf {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("sortie");
    let ext = if format.to_lowercase() == "png" {
        "png"
    } else {
        "jpg"
    };
    PathBuf::from(parent).join(format!("{stem}.{ext}"))
}

fn normalized_output_path(path: PathBuf, format: &str) -> PathBuf {
    let extension = if format.eq_ignore_ascii_case("png") {
        "png"
    } else {
        "jpg"
    };
    path.with_extension(extension)
}

fn update_output_extension(ui: &MainWindow) {
    let format = ui.get_output_format().as_str().trim().to_owned();
    let output = ui.get_output_path().as_str().trim().to_owned();
    if output.is_empty() {
        return;
    }

    let original = PathBuf::from(output.clone());
    let updated = normalized_output_path(PathBuf::from(output), &format);
    if updated != original {
        ui.set_output_path(updated.display().to_string().into());
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn open_output_folder_gui(ui: &MainWindow, toaster: &Toaster) {
    let path = ui.get_output_path().as_str().to_owned();
    if path.is_empty() {
        return;
    }
    let path = PathBuf::from(path);
    let target = if path.is_dir() {
        path
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    if let Err(error) = open_system_path(&target) {
        set_status(ui, toaster, format!("Ouverture impossible : {error}"), 2);
    }
}

fn open_output_file_gui(ui: &MainWindow, toaster: &Toaster) {
    let path = ui.get_output_path().as_str().to_owned();
    if path.is_empty() {
        return;
    }
    let path = PathBuf::from(path);
    if let Err(error) = open_system_path(&path) {
        set_status(ui, toaster, format!("Ouverture impossible : {error}"), 2);
    }
}

fn open_system_path(path: &Path) -> Result<()> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.arg("/C").arg("start").arg("").arg(path);
        command
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(path);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    command
        .spawn()
        .context("impossible de lancer l'outil d'ouverture systeme")?;
    Ok(())
}
