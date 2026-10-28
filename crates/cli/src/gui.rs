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

fn convert_gui(ui: &MainWindow, toaster: &Toaster) {
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

    let options = ConversionOptions {
        format,
        apply_transforms: true,
        max_pixels: 1 << 28,
    };

    match convert_file(&input_path, &output, &options) {
        Ok(result) => {
            ui.set_output_path(output.display().to_string().into());
            ui.set_output_available(true);
            let status = format!("Conversion terminee ({}x{}).", result.width, result.height);
            set_report(
                ui,
                "Resultat",
                format!(
                    "HEIC Converter - resultat\nFichier source : {}\nFichier genere : {}\nDimensions : {} x {}\nTaille : {} octets\nAlpha : {}\n16 bits : {}",
                    result.input.display(),
                    result.output.display(),
                    result.width,
                    result.height,
                    result.output_bytes,
                    result.has_alpha,
                    result.sixteen_bit
                ),
                status,
                0,
                toaster,
            );
        }
        Err(error) => {
            set_status(ui, toaster, format!("Conversion impossible : {error}"), 2);
        }
    }
}

fn set_report(
    ui: &MainWindow,
    title: impl Into<slint::SharedString>,
    report: String,
    status: impl Into<slint::SharedString>,
    level: i32,
    toaster: &Toaster,
) {
    ui.set_report_title(title.into());
    ui.set_report(report.into());
    ui.set_report_available(true);
    set_status(ui, toaster, status, level);
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
    let target = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
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
