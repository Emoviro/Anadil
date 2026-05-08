use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anadil::{check_source, diagnostics::Diagnostic, run_source_diagnostic};
use eframe::egui::{
    self, text::LayoutJob, Color32, FontFamily, FontId, RichText, ScrollArea, TextEdit, TextFormat,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Anadil IDE")
            .with_inner_size([1220.0, 780.0])
            .with_min_inner_size([920.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Anadil IDE",
        options,
        Box::new(|context| {
            configure_fonts(&context.egui_ctx);
            Ok(Box::new(AnadilIde::default()))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Output,
    Diagnostics,
}

#[derive(Debug)]
struct AnadilIde {
    source: String,
    saved_source: String,
    current_path: String,
    status: String,
    output: String,
    diagnostics: Vec<Diagnostic>,
    examples: Vec<PathBuf>,
    selected_tab: Tab,
    build_exe: Option<String>,
}

impl Default for AnadilIde {
    fn default() -> Self {
        let source = starter_source();
        Self {
            saved_source: source.clone(),
            source,
            current_path: "adsiz.ana".to_string(),
            status: "Hazir".to_string(),
            output: "Henüz calistirma yok.".to_string(),
            diagnostics: Vec::new(),
            examples: list_examples(),
            selected_tab: Tab::Output,
            build_exe: None,
        }
    }
}

impl eframe::App for AnadilIde {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ui.ctx());
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        self.top_bar(ui);
        self.left_panel(ui);
        self.bottom_panel(ui);
        self.editor_panel(ui);
    }
}

impl AnadilIde {
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar")
            .exact_size(52.0)
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Anadil IDE").strong().size(20.0));
                    ui.separator();

                    if ui
                        .button("Ac")
                        .on_hover_text("Ctrl+O ile dosya sec")
                        .clicked()
                    {
                        self.open_file_dialog();
                    }
                    if ui.button("Kaydet").on_hover_text("Ctrl+S").clicked() {
                        self.save_current_path();
                    }
                    if ui.button("Farkli Kaydet").clicked() {
                        self.save_as_dialog();
                    }

                    ui.separator();

                    if ui.button("Kontrol").clicked() {
                        self.check();
                    }
                    if ui.button("Calistir").on_hover_text("F5").clicked() {
                        self.run();
                    }
                    if ui.button("EXE Derle").on_hover_text("Ctrl+B").clicked() {
                        self.build();
                    }

                    ui.separator();
                    let dirty = if self.is_dirty() {
                        "Degisiklik var"
                    } else {
                        "Kayitli"
                    };
                    ui.label(
                        RichText::new(format!("{} - {dirty}", self.status))
                            .color(Color32::from_rgb(178, 190, 181)),
                    );
                });
            });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("left_panel")
            .resizable(true)
            .default_size(255.0)
            .size_range(210.0..=360.0)
            .show_inside(ui, |ui| {
                ui.heading("Dosya");
                ui.label("Aktif yol");
                ui.add(
                    TextEdit::singleline(&mut self.current_path)
                        .desired_width(f32::INFINITY)
                        .hint_text("examples\\topla.ana"),
                );
                if ui.button("Bu yoldan ac").clicked() {
                    self.open_current_path();
                }

                ui.add_space(12.0);
                ui.heading("Ornekler");
                ui.add_space(4.0);

                ScrollArea::vertical().show(ui, |ui| {
                    let examples = self.examples.clone();
                    for path in examples {
                        let name = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("ornek.ana");
                        let selected = self.current_path == path.display().to_string();
                        if ui.selectable_label(selected, name).clicked() {
                            self.load_path(&path);
                        }
                    }
                });
            });
    }

    fn editor_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.display_path()).strong());
                if let Some(exe) = &self.build_exe {
                    ui.separator();
                    ui.label(
                        RichText::new(format!("EXE: {exe}"))
                            .color(Color32::from_rgb(135, 214, 150)),
                    );
                }
            });

            ui.add_space(6.0);

            let diagnostics = self.diagnostics.clone();
            let mut layouter =
                move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    let mut job = highlight_job(text.as_str(), &diagnostics);
                    job.wrap.max_width = wrap_width;
                    ui.fonts_mut(|fonts| fonts.layout_job(job))
                };

            let response = ui.add(
                TextEdit::multiline(&mut self.source)
                    .font(FontId::new(15.0, FontFamily::Monospace))
                    .desired_width(f32::INFINITY)
                    .desired_rows(26)
                    .lock_focus(true)
                    .layouter(&mut layouter),
            );

            if response.changed() {
                self.build_exe = None;
                self.status = "Degisiklik var".to_string();
                self.check_silent();
            }
        });
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_panel")
            .resizable(true)
            .default_size(210.0)
            .size_range(140.0..=360.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.selected_tab == Tab::Output, "Cikti")
                        .clicked()
                    {
                        self.selected_tab = Tab::Output;
                    }
                    if ui
                        .selectable_label(self.selected_tab == Tab::Diagnostics, "Diagnostics")
                        .clicked()
                    {
                        self.selected_tab = Tab::Diagnostics;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} diagnostic", self.diagnostics.len()));
                    });
                });
                ui.separator();

                match self.selected_tab {
                    Tab::Output => {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.output)
                                    .font(FontId::new(13.5, FontFamily::Monospace))
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(8)
                                    .interactive(false),
                            );
                        });
                    }
                    Tab::Diagnostics => self.diagnostics_ui(ui),
                }
            });
    }

    fn diagnostics_ui(&mut self, ui: &mut egui::Ui) {
        if self.diagnostics.is_empty() {
            ui.label(RichText::new("Hata yok.").color(Color32::from_rgb(171, 186, 174)));
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            for diagnostic in &self.diagnostics {
                let place = diagnostic
                    .span
                    .map(|span| format!("satir {}, sutun {}", span.line, span.column))
                    .unwrap_or_else(|| "konum yok".to_string());

                ui.group(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} / {}",
                            diagnostic.stage.as_str(),
                            diagnostic.severity.as_str()
                        ))
                        .strong()
                        .color(Color32::from_rgb(255, 178, 181)),
                    );
                    ui.label(RichText::new(place).color(Color32::from_rgb(171, 186, 174)));
                    ui.label(&diagnostic.message);
                });
            }
        });
    }

    fn check(&mut self) {
        self.check_silent();
        self.selected_tab = Tab::Diagnostics;
        self.status = if self.diagnostics.is_empty() {
            "Program gecerli".to_string()
        } else {
            "Hata bulundu".to_string()
        };
    }

    fn check_silent(&mut self) {
        self.diagnostics = match check_source(&self.source) {
            Ok(()) => Vec::new(),
            Err(diagnostic) => vec![diagnostic],
        };
    }

    fn run(&mut self) {
        match run_source_diagnostic(&self.source) {
            Ok(output) => {
                self.output = if output.is_empty() {
                    "Program cikti uretmedi.".to_string()
                } else {
                    output
                };
                self.diagnostics.clear();
                self.status = "Calistirildi".to_string();
                self.selected_tab = Tab::Output;
            }
            Err(diagnostic) => {
                self.output.clear();
                self.diagnostics = vec![diagnostic];
                self.status = "Calisma zamani hatasi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn build(&mut self) {
        self.check_silent();
        if !self.diagnostics.is_empty() {
            self.status = "Build once compile hatasini duzeltmeli".to_string();
            self.selected_tab = Tab::Diagnostics;
            return;
        }

        match write_ide_source(&self.source).and_then(|path| run_native_build(&path)) {
            Ok(exe) => {
                self.build_exe = Some(exe.clone());
                self.output = format!("Native executable uretildi:\n{exe}");
                self.status = "EXE derlendi".to_string();
                self.selected_tab = Tab::Output;
            }
            Err(message) => {
                self.diagnostics = vec![Diagnostic::native(message)];
                self.status = "Native build hatasi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn open_current_path(&mut self) {
        let path = PathBuf::from(self.current_path.trim());
        self.load_path(&path);
    }

    fn load_path(&mut self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(source) => {
                self.source = source;
                self.saved_source = self.source.clone();
                self.current_path = path.display().to_string();
                self.output = "Dosya acildi.".to_string();
                self.status = "Dosya acildi".to_string();
                self.build_exe = None;
                self.check_silent();
            }
            Err(error) => {
                self.diagnostics = vec![Diagnostic::io(format!(
                    "Dosya okunamadi `{}`: {error}",
                    path.display()
                ))];
                self.status = "Dosya okunamadi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn save_current_path(&mut self) {
        if self.current_path_is_placeholder() {
            self.save_as_dialog();
            return;
        }

        let path = PathBuf::from(self.current_path.trim());
        match fs::write(&path, &self.source) {
            Ok(()) => {
                self.saved_source = self.source.clone();
                self.status = "Kaydedildi".to_string();
                self.output = format!("Kaydedildi:\n{}", path.display());
            }
            Err(error) => {
                self.diagnostics = vec![Diagnostic::io(format!(
                    "Dosya yazilamadi `{}`: {error}",
                    path.display()
                ))];
                self.status = "Kaydedilemedi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Anadil kaynak", &["ana"])
            .add_filter("Metin", &["txt"])
            .set_directory(".")
            .pick_file()
        {
            self.load_path(&path);
        }
    }

    fn save_as_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Anadil kaynak", &["ana"])
            .set_file_name(default_save_name(&self.current_path))
            .save_file()
        {
            self.current_path = path.display().to_string();
            self.save_current_path();
        }
    }

    fn handle_shortcuts(&mut self, context: &egui::Context) {
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            self.open_file_dialog();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            self.save_current_path();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.run();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::B)) {
            self.build();
        }
    }

    fn is_dirty(&self) -> bool {
        self.source != self.saved_source
    }

    fn current_path_is_placeholder(&self) -> bool {
        let path = self.current_path.trim();
        path.is_empty() || path == "adsiz.ana"
    }

    fn display_path(&self) -> String {
        if self.is_dirty() {
            format!("{} *", self.current_path)
        } else {
            self.current_path.clone()
        }
    }

    fn window_title(&self) -> String {
        if self.is_dirty() {
            format!("Anadil IDE - {} *", self.current_path)
        } else {
            format!("Anadil IDE - {}", self.current_path)
        }
    }
}

fn configure_fonts(context: &egui::Context) {
    let mut style = (*context.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(22, 25, 23);
    style.visuals.window_fill = Color32::from_rgb(25, 29, 27);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 38, 34);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 54, 47);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(38, 74, 47);
    context.set_global_style(style);
}

fn default_save_name(path: &str) -> &str {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "adsiz.ana" {
        "adsiz.ana"
    } else {
        Path::new(trimmed)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("adsiz.ana")
    }
}

fn list_examples() -> Vec<PathBuf> {
    let mut examples = fs::read_dir("examples")
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ana"))
        .collect::<Vec<_>>();

    examples.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    examples
}

fn write_ide_source(source: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from("target").join("ide-native");
    fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "IDE build klasoru olusturulamadi `{}`: {error}",
            dir.display()
        )
    })?;

    let path = dir.join("current.ana");
    fs::write(&path, source).map_err(|error| {
        format!(
            "IDE kaynak dosyasi yazilamadi `{}`: {error}",
            path.display()
        )
    })?;
    Ok(path)
}

fn run_native_build(path: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("anadil")
        .arg("--")
        .arg("derle")
        .arg("--json")
        .arg(path)
        .output()
        .map_err(|error| format!("Native build komutu calistirilamadi: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        if let Some(message) = extract_json_string(&stdout, "message") {
            return Err(message);
        }
        return Err(stderr.trim().to_string());
    }

    extract_json_string(&stdout, "exe").ok_or_else(|| stdout.trim().to_string())
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = json.find(&marker)? + marker.len();
    let mut out = String::new();
    let mut escaped = false;

    for ch in json[start..].chars() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }

    None
}

fn highlight_job(source: &str, diagnostics: &[Diagnostic]) -> LayoutJob {
    let mut job = LayoutJob::default();
    let error_lines = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.span.map(|span| span.line))
        .collect::<Vec<_>>();

    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let background = if error_lines.contains(&line_number) {
            Color32::from_rgb(54, 28, 31)
        } else {
            Color32::TRANSPARENT
        };

        append_highlighted_line(&mut job, line, background);
        job.append(
            "\n",
            0.0,
            format(Color32::from_rgb(230, 238, 231), background),
        );
    }

    if source.is_empty() {
        job.append(
            "",
            0.0,
            format(Color32::from_rgb(230, 238, 231), Color32::TRANSPARENT),
        );
    }

    job
}

fn append_highlighted_line(job: &mut LayoutJob, line: &str, background: Color32) {
    let mut index = 0;
    let chars = line.char_indices().collect::<Vec<_>>();

    while index < line.len() {
        let rest = &line[index..];
        if rest.starts_with("//") {
            job.append(
                rest,
                0.0,
                format(Color32::from_rgb(120, 135, 124), background),
            );
            break;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };

        if ch == '"' {
            let end = string_end(rest);
            let token = &rest[..end];
            job.append(
                token,
                0.0,
                format(Color32::from_rgb(239, 166, 106), background),
            );
            index += token.len();
        } else if ch.is_ascii_digit() {
            let end = take_while(rest, |c| c.is_ascii_digit());
            let token = &rest[..end];
            job.append(
                token,
                0.0,
                format(Color32::from_rgb(232, 193, 90), background),
            );
            index += token.len();
        } else if is_ident_start(ch) {
            let end = take_while(rest, is_ident_continue);
            let token = &rest[..end];
            job.append(token, 0.0, token_format(token, background));
            index += token.len();
        } else {
            let next = chars
                .iter()
                .find_map(|(char_index, _)| (*char_index > index).then_some(*char_index))
                .unwrap_or(line.len());
            let token = &line[index..next];
            job.append(
                token,
                0.0,
                format(Color32::from_rgb(210, 222, 213), background),
            );
            index = next;
        }
    }
}

fn string_end(rest: &str) -> usize {
    let mut escaped = false;
    for (index, ch) in rest.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return index + ch.len_utf8(),
            _ => {}
        }
    }
    rest.len()
}

fn take_while(rest: &str, predicate: impl Fn(char) -> bool) -> usize {
    rest.char_indices()
        .find_map(|(index, ch)| (!predicate(ch)).then_some(index))
        .unwrap_or(rest.len())
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn token_format(token: &str, background: Color32) -> TextFormat {
    if matches!(
        token,
        "eğer" | "değilse" | "döngü" | "kır" | "devam" | "dön" | "doğru" | "yanlış"
    ) {
        return format(Color32::from_rgb(123, 216, 143), background);
    }

    if matches!(token, "sayı" | "mantık" | "metin") {
        return format(Color32::from_rgb(101, 199, 208), background);
    }

    if matches!(token, "Ana" | "yazdir") {
        return format(Color32::from_rgb(166, 215, 255), background);
    }

    if token.chars().next().is_some_and(char::is_uppercase) {
        return format(Color32::from_rgb(217, 233, 140), background);
    }

    format(Color32::from_rgb(230, 238, 231), background)
}

fn format(color: Color32, background: Color32) -> TextFormat {
    TextFormat {
        font_id: FontId::new(15.0, FontFamily::Monospace),
        color,
        background,
        ..Default::default()
    }
}

fn starter_source() -> String {
    "\
Topla(a: sayı, b: sayı) -> sayı {
    dön a + b;
}

Ana() {
    sonuc: sayı = Topla(10, 20);
    yazdir(sonuc);
}
"
    .to_string()
}
