use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use anadil::{check_source, diagnostics::Diagnostic, run_source_diagnostic};
use eframe::egui::{
    self, text::LayoutJob, text_edit::TextEditState, Color32, FontFamily, FontId, RichText,
    ScrollArea, TextEdit, TextFormat,
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
            Ok(Box::new(AnadilIde::new()))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Output,
    Diagnostics,
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Interpret,
    Compile,
    Compare,
}

impl RunMode {
    fn label(self) -> &'static str {
        match self {
            Self::Interpret => "Interpret et",
            Self::Compile => "Compile et",
            Self::Compare => "Karsilastir",
        }
    }
}

#[derive(Debug)]
struct AnadilIde {
    source: String,
    saved_source: String,
    current_path: String,
    project_root: Option<PathBuf>,
    project_files: Vec<PathBuf>,
    status: String,
    output: String,
    build_output: String,
    diagnostics: Vec<Diagnostic>,
    examples: Vec<PathBuf>,
    selected_tab: Tab,
    build_exe: Option<String>,
    run_mode: RunMode,
    new_file_name: String,
    rename_file_name: String,
    selected_diagnostic: Option<usize>,
    pending_editor_jump: Option<(usize, usize)>,
    pending_editor_scroll_line: Option<usize>,
}

impl Default for AnadilIde {
    fn default() -> Self {
        let source = starter_source();
        Self {
            saved_source: source.clone(),
            source,
            current_path: "adsiz.ana".to_string(),
            project_root: None,
            project_files: Vec::new(),
            status: "Hazir".to_string(),
            output: "Henüz calistirma yok.".to_string(),
            build_output: "Henuz build yok.".to_string(),
            diagnostics: Vec::new(),
            examples: list_examples(),
            selected_tab: Tab::Output,
            build_exe: None,
            run_mode: RunMode::Interpret,
            new_file_name: "yeni.ana".to_string(),
            rename_file_name: "adsiz.ana".to_string(),
            selected_diagnostic: None,
            pending_editor_jump: None,
            pending_editor_scroll_line: None,
        }
    }
}

impl eframe::App for AnadilIde {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        self.handle_shortcuts(&context);
        context.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        egui::Panel::top("top_bar")
            .exact_size(46.0)
            .frame(panel_frame(Color32::from_rgb(37, 37, 38), 8, 4))
            .show_inside(ui, |ui| self.top_bar(ui));

        egui::Panel::left("left_panel")
            .resizable(true)
            .show_separator_line(true)
            .default_size(285.0)
            .size_range(220.0..=390.0)
            .frame(panel_frame(Color32::from_rgb(37, 37, 38), 8, 8))
            .show_inside(ui, |ui| self.left_panel(ui));

        egui::Panel::bottom("bottom_panel")
            .resizable(true)
            .show_separator_line(true)
            .default_size(210.0)
            .size_range(140.0..=360.0)
            .frame(panel_frame(Color32::from_rgb(30, 30, 30), 8, 6))
            .show_inside(ui, |ui| self.bottom_panel(ui));

        egui::CentralPanel::default()
            .frame(panel_frame(Color32::from_rgb(30, 30, 30), 10, 8))
            .show_inside(ui, |ui| self.editor_panel(ui));
    }
}

impl AnadilIde {
    fn new() -> Self {
        let mut ide = Self::default();
        ide.restore_last_session();
        ide
    }

    fn restore_last_session(&mut self) {
        let Some(state) = load_ide_state() else {
            return;
        };

        if let Some(root) = state.project_root.filter(|path| path.is_dir()) {
            self.project_root = Some(root);
            self.refresh_project_files();
        }

        if let Some(path) = state.current_path.filter(|path| path.is_file()) {
            self.load_path(&path);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Anadil IDE")
                    .strong()
                    .size(18.0)
                    .color(Color32::from_rgb(220, 224, 229)),
            );
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
            if ui.button("Klasor Ac").clicked() {
                self.open_folder_dialog();
            }

            ui.separator();

            if ui.button("Kontrol").clicked() {
                self.check();
            }

            egui::ComboBox::from_id_salt("run_mode")
                .selected_text(self.run_mode.label())
                .width(118.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.run_mode,
                        RunMode::Interpret,
                        RunMode::Interpret.label(),
                    );
                    ui.selectable_value(
                        &mut self.run_mode,
                        RunMode::Compile,
                        RunMode::Compile.label(),
                    );
                    ui.selectable_value(
                        &mut self.run_mode,
                        RunMode::Compare,
                        RunMode::Compare.label(),
                    );
                });

            if ui.button("Yap").on_hover_text("F5").clicked() {
                self.run_selected_mode();
            }
            if ui
                .add_enabled(self.build_exe.is_some(), egui::Button::new("EXE Calistir"))
                .on_hover_text("Ctrl+Shift+F5")
                .clicked()
            {
                self.run_built_exe();
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
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("EXPLORER")
                .strong()
                .small()
                .color(Color32::from_rgb(204, 210, 218)),
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui
                .button("Klasor")
                .on_hover_text("Proje klasoru ac")
                .clicked()
            {
                self.open_folder_dialog();
            }
            if ui.button("Yeni").clicked() {
                self.new_file();
            }
            if ui
                .add_enabled(self.project_root.is_some(), egui::Button::new("Yenile"))
                .clicked()
            {
                self.refresh_project_files();
            }
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new(self.project_root_label())
                .small()
                .color(Color32::from_rgb(156, 163, 175)),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.new_file_name)
                    .desired_width(f32::INFINITY)
                    .hint_text("yeni.ana"),
            );
            if ui
                .add_enabled(self.project_root.is_some(), egui::Button::new("Olustur"))
                .clicked()
            {
                self.create_project_file();
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new("DOSYALAR")
                .strong()
                .small()
                .color(Color32::from_rgb(204, 210, 218)),
        );

        let file_list_height = (ui.available_height() - 190.0).max(180.0);
        ScrollArea::vertical()
            .id_salt("project_files")
            .max_height(file_list_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.project_root.is_none() {
                    ui.label(
                        RichText::new("Bir proje klasoru ac.")
                            .color(Color32::from_rgb(171, 186, 174)),
                    );
                } else if self.project_files.is_empty() {
                    ui.label(
                        RichText::new("Bu klasorde .ana dosyasi yok.")
                            .color(Color32::from_rgb(171, 186, 174)),
                    );
                }

                let files = self.project_files.clone();
                for path in files {
                    self.project_file_row(ui, &path);
                }
            });

        ui.add_space(8.0);
        ui.separator();

        egui::CollapsingHeader::new("Aktif dosya")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Aktif yol");
                ui.add(
                    TextEdit::singleline(&mut self.current_path)
                        .desired_width(f32::INFINITY)
                        .hint_text("examples\\topla.ana"),
                );
                if ui.button("Bu yoldan ac").clicked() {
                    self.open_current_path();
                }

                ui.add_space(8.0);
                ui.label("Yeni ad");
                ui.add(
                    TextEdit::singleline(&mut self.rename_file_name)
                        .desired_width(f32::INFINITY)
                        .hint_text("ornek.ana"),
                );
                ui.horizontal(|ui| {
                    let has_real_file = !self.current_path_is_placeholder();
                    if ui
                        .add_enabled(has_real_file, egui::Button::new("Yeniden Adlandir"))
                        .clicked()
                    {
                        self.rename_current_file();
                    }
                    if ui
                        .add_enabled(has_real_file, egui::Button::new("Sil"))
                        .clicked()
                    {
                        self.delete_current_file();
                    }
                });
            });

        egui::CollapsingHeader::new("Ornekler")
            .default_open(false)
            .show(ui, |ui| {
                ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                    let examples = self.examples.clone();
                    for path in examples {
                        let name = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("ornek.ana");
                        let selected = self.current_path == path.display().to_string();
                        if ui.selectable_label(selected, name).clicked() {
                            self.open_path_with_guard(&path);
                        }
                    }
                });
            });
    }

    fn project_file_row(&mut self, ui: &mut egui::Ui, path: &Path) {
        let relative = self.relative_project_path(path);
        let depth = relative_component_depth(&relative).saturating_sub(1);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(relative.as_str())
            .to_string();
        let selected = self.current_path == path.display().to_string();
        let hover_text = relative.clone();
        let label = if depth == 0 {
            name
        } else {
            format!("{name}  {}", parent_hint(&relative))
        };

        ui.horizontal(|ui| {
            ui.add_space(depth as f32 * 12.0);
            let color = if selected {
                Color32::from_rgb(220, 224, 229)
            } else {
                Color32::from_rgb(200, 204, 209)
            };

            if ui
                .selectable_label(selected, RichText::new(label).color(color))
                .on_hover_text(hover_text)
                .clicked()
            {
                self.open_path_with_guard(path);
            }
        });
    }

    fn editor_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.display_path())
                    .strong()
                    .color(Color32::from_rgb(220, 224, 229)),
            );
            if let Some(exe) = &self.build_exe {
                ui.separator();
                ui.label(
                    RichText::new(format!("EXE: {exe}")).color(Color32::from_rgb(135, 214, 150)),
                );
            }
        });

        ui.add_space(6.0);
        self.apply_pending_editor_jump(ui);

        let diagnostics = self.diagnostics.clone();
        let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = highlight_job(text.as_str(), &diagnostics);
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        let editor_id = ui.make_persistent_id("source_editor");
        let line_count = editor_line_count(&self.source);
        let active_line = self.active_diagnostic_line();
        let scroll_line = self.pending_editor_scroll_line.take();

        let output = ScrollArea::vertical()
            .id_salt("source_editor_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    self.line_numbers_ui(ui, line_count, active_line, scroll_line);
                    ui.add_space(6.0);
                    TextEdit::multiline(&mut self.source)
                        .id(editor_id)
                        .font(FontId::new(15.0, FontFamily::Monospace))
                        .desired_width(f32::INFINITY)
                        .desired_rows(line_count.max(26))
                        .lock_focus(true)
                        .layouter(&mut layouter)
                        .show(ui)
                })
                .inner
            })
            .inner;

        if output.response.changed() {
            self.build_exe = None;
            self.build_output = "Build sonucu guncel degil.".to_string();
            self.status = "Degisiklik var".to_string();
            self.check_silent();
        }
    }

    fn apply_pending_editor_jump(&mut self, ui: &mut egui::Ui) {
        let Some((line, column)) = self.pending_editor_jump.take() else {
            return;
        };

        let editor_id = ui.make_persistent_id("source_editor");
        let char_index = char_index_for_line_column(&self.source, line, column);
        let mut state = TextEditState::load(ui.ctx(), editor_id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(char_index),
            )));
        state.store(ui.ctx(), editor_id);
        ui.ctx()
            .memory_mut(|memory| memory.request_focus(editor_id));
        self.status = format!("Diagnostic konumu: satir {line}, sutun {column}");
    }

    fn line_numbers_ui(
        &self,
        ui: &mut egui::Ui,
        line_count: usize,
        active_line: Option<usize>,
        scroll_line: Option<usize>,
    ) {
        let digits = line_count.to_string().len().max(2);
        let width = (digits as f32 * 8.0) + 18.0;

        egui::Frame::new()
            .fill(Color32::from_rgb(30, 30, 30))
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.set_min_width(width);
                for line in 1..=line_count {
                    let selected = active_line == Some(line);
                    let color = if selected {
                        Color32::from_rgb(220, 224, 229)
                    } else {
                        Color32::from_rgb(110, 116, 126)
                    };
                    let fill = if selected {
                        Color32::from_rgb(55, 65, 81)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let response = egui::Frame::new().fill(fill).show(ui, |ui| {
                        ui.set_min_width(width);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(line.to_string())
                                    .font(FontId::new(15.0, FontFamily::Monospace))
                                    .color(color),
                            );
                        });
                    });

                    if scroll_line == Some(line) {
                        response.response.scroll_to_me(Some(egui::Align::Center));
                    }
                }
            });
    }

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
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
            if ui
                .selectable_label(self.selected_tab == Tab::Build, "Build")
                .clicked()
            {
                self.selected_tab = Tab::Build;
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
            Tab::Build => {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.build_output)
                            .font(FontId::new(13.5, FontFamily::Monospace))
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .interactive(false),
                    );
                });
            }
        }
    }

    fn diagnostics_ui(&mut self, ui: &mut egui::Ui) {
        if self.diagnostics.is_empty() {
            ui.label(RichText::new("Hata yok.").color(Color32::from_rgb(171, 186, 174)));
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            let diagnostics = self.diagnostics.clone();
            for (index, diagnostic) in diagnostics.iter().enumerate() {
                let place = diagnostic
                    .span
                    .map(|span| format!("satir {}, sutun {}", span.line, span.column))
                    .unwrap_or_else(|| "konum yok".to_string());

                let fill = if self.selected_diagnostic == Some(index) {
                    Color32::from_rgb(42, 55, 47)
                } else {
                    Color32::from_rgb(31, 36, 33)
                };

                let card = egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
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
                    if diagnostic.span.is_some() {
                        ui.label(
                            RichText::new("Tikla: editor konumuna git")
                                .small()
                                .color(Color32::from_rgb(135, 214, 150)),
                        );
                    }
                });

                let response = card.response.interact(egui::Sense::click());
                if diagnostic.span.is_some() {
                    response.clone().on_hover_text("Editor konumuna git");
                }

                if response.clicked() {
                    self.jump_to_diagnostic(index);
                }
            }
        });
    }

    fn jump_to_diagnostic(&mut self, index: usize) {
        let Some(diagnostic) = self.diagnostics.get(index) else {
            return;
        };

        let Some(span) = diagnostic.span else {
            self.status = "Bu diagnostic icin kaynak konumu yok".to_string();
            return;
        };

        self.selected_diagnostic = Some(index);
        self.pending_editor_jump = Some((span.line, span.column));
        self.pending_editor_scroll_line = Some(span.line);
    }

    fn active_diagnostic_line(&self) -> Option<usize> {
        self.selected_diagnostic
            .and_then(|index| self.diagnostics.get(index))
            .and_then(|diagnostic| diagnostic.span)
            .map(|span| span.line)
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
        self.selected_diagnostic = None;
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

    fn run_selected_mode(&mut self) {
        match self.run_mode {
            RunMode::Interpret => self.run(),
            RunMode::Compile => self.build(),
            RunMode::Compare => self.compare_interpreter_and_native(),
        }
    }

    fn build(&mut self) {
        self.check_silent();
        if !self.diagnostics.is_empty() {
            self.status = "Build once compile hatasini duzeltmeli".to_string();
            self.selected_tab = Tab::Diagnostics;
            return;
        }

        let Some(path) = self.prepare_build_source_path() else {
            return;
        };

        match run_native_build(&path) {
            Ok(exe) => {
                self.build_exe = Some(exe.clone());
                self.build_output = format!("Native executable uretildi:\n{exe}");
                self.status = "EXE derlendi".to_string();
                self.selected_tab = Tab::Build;
            }
            Err(message) => {
                self.build_output = message.clone();
                self.diagnostics = vec![Diagnostic::native(message)];
                self.status = "Native build hatasi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn run_built_exe(&mut self) {
        let Some(exe) = self.build_exe.clone() else {
            self.build_output = "Once EXE Derle ile native executable uret.".to_string();
            self.status = "Calistirilacak EXE yok".to_string();
            self.selected_tab = Tab::Build;
            return;
        };

        let exe_path = PathBuf::from(&exe);
        match run_executable(&exe_path) {
            Ok(output) => {
                self.build_output = format_exe_run_output(&exe_path, &output);
                if output.status.success() {
                    self.status = "EXE calistirildi".to_string();
                    self.diagnostics.clear();
                } else {
                    let code = exit_code_label(&output.status);
                    self.status = "EXE hata ile bitti".to_string();
                    self.diagnostics = vec![Diagnostic::native(format!(
                        "Native executable basarisiz bitti: {code}"
                    ))];
                }
                self.selected_tab = Tab::Build;
            }
            Err(error) => {
                let message = format!("Native executable calistirilamadi `{}`: {error}", exe);
                self.build_output = message.clone();
                self.diagnostics = vec![Diagnostic::native(message)];
                self.status = "EXE calistirilamadi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn compare_interpreter_and_native(&mut self) {
        let interpreter_output = match run_source_diagnostic(&self.source) {
            Ok(output) => output,
            Err(diagnostic) => {
                self.output.clear();
                self.diagnostics = vec![diagnostic];
                self.status = "Interpreter hatasi".to_string();
                self.selected_tab = Tab::Diagnostics;
                return;
            }
        };

        if self.build_exe.is_none() || self.is_dirty() {
            self.build();
        }

        let Some(exe) = self.build_exe.clone() else {
            if self.selected_tab != Tab::Diagnostics {
                self.status = "Karsilastirma iptal edildi".to_string();
                self.selected_tab = Tab::Build;
            }
            return;
        };

        let exe_path = PathBuf::from(&exe);
        match run_executable(&exe_path) {
            Ok(native_output) => {
                let native_stdout = String::from_utf8_lossy(&native_output.stdout);
                let native_stderr = String::from_utf8_lossy(&native_output.stderr);
                let interpreter_text = interpreter_output.trim_end();
                let native_text = native_stdout.trim_end();

                self.build_output = format_comparison_output(
                    interpreter_text,
                    native_text,
                    native_stderr.trim_end(),
                    &native_output.status,
                );
                self.selected_tab = Tab::Build;

                if native_output.status.success() && interpreter_text == native_text {
                    self.status = "Interpreter/native ayni".to_string();
                    self.diagnostics.clear();
                } else {
                    let message = if native_output.status.success() {
                        "Interpreter ve native ciktilari farkli".to_string()
                    } else {
                        format!(
                            "Native executable basarisiz bitti: {}",
                            exit_code_label(&native_output.status)
                        )
                    };
                    self.status = "Karsilastirma fark buldu".to_string();
                    self.diagnostics = vec![Diagnostic::native(message)];
                }
            }
            Err(error) => {
                let message = format!("Native executable calistirilamadi `{}`: {error}", exe);
                self.build_output = message.clone();
                self.diagnostics = vec![Diagnostic::native(message)];
                self.status = "Karsilastirma calisamadi".to_string();
                self.selected_tab = Tab::Diagnostics;
            }
        }
    }

    fn prepare_build_source_path(&mut self) -> Option<PathBuf> {
        if self.current_path_is_placeholder() || self.is_dirty() {
            self.status = "Build icin once kaydediliyor".to_string();
            if !self.save_current_path() {
                self.status = "Build iptal edildi".to_string();
                self.build_output =
                    "Build icin dosya kaydedilemedi veya islem iptal edildi.".to_string();
                self.selected_tab = Tab::Build;
                return None;
            }
        }

        Some(PathBuf::from(self.current_path.trim()))
    }

    fn open_current_path(&mut self) {
        let path = PathBuf::from(self.current_path.trim());
        self.open_path_with_guard(&path);
    }

    fn open_path_with_guard(&mut self, path: &Path) {
        if self.confirm_discard_unsaved() {
            self.load_path(path);
        }
    }

    fn load_path(&mut self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(source) => {
                self.source = source;
                self.saved_source = self.source.clone();
                self.current_path = path.display().to_string();
                self.rename_file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("adsiz.ana")
                    .to_string();
                self.output = "Dosya acildi.".to_string();
                self.status = "Dosya acildi".to_string();
                self.build_exe = None;
                self.build_output = "Henuz build yok.".to_string();
                self.ensure_project_root_for_file(path);
                self.save_ide_state();
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

    fn save_current_path(&mut self) -> bool {
        if self.current_path_is_placeholder() {
            return self.save_as_dialog();
        }

        let path = PathBuf::from(self.current_path.trim());
        match fs::write(&path, &self.source) {
            Ok(()) => {
                self.saved_source = self.source.clone();
                self.status = "Kaydedildi".to_string();
                self.output = format!("Kaydedildi:\n{}", path.display());
                self.ensure_project_root_for_file(&path);
                self.refresh_project_files();
                self.save_ide_state();
                true
            }
            Err(error) => {
                self.diagnostics = vec![Diagnostic::io(format!(
                    "Dosya yazilamadi `{}`: {error}",
                    path.display()
                ))];
                self.status = "Kaydedilemedi".to_string();
                self.selected_tab = Tab::Diagnostics;
                false
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if !self.confirm_discard_unsaved() {
            return;
        }

        let mut dialog = rfd::FileDialog::new()
            .add_filter("Anadil kaynak", &["ana"])
            .add_filter("Metin", &["txt"])
            .set_directory(".");

        if let Some(root) = &self.project_root {
            dialog = dialog.set_directory(root);
        }

        if let Some(path) = dialog.pick_file() {
            self.load_path(&path);
        }
    }

    fn save_as_dialog(&mut self) -> bool {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Anadil kaynak", &["ana"])
            .set_file_name(default_save_name(&self.current_path));

        if let Some(root) = &self.project_root {
            dialog = dialog.set_directory(root);
        }

        if let Some(path) = dialog.save_file() {
            self.current_path = path.display().to_string();
            self.rename_file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("adsiz.ana")
                .to_string();
            return self.save_current_path();
        }

        false
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_directory(".").pick_folder() {
            self.project_root = Some(path.clone());
            self.refresh_project_files();
            self.save_ide_state();
            self.status = "Proje acildi".to_string();
            self.output = format!("Proje klasoru acildi:\n{}", path.display());
        }
    }

    fn refresh_project_files(&mut self) {
        if let Some(root) = &self.project_root {
            self.project_files = list_project_files(root);
        }
    }

    fn new_file(&mut self) {
        if !self.confirm_discard_unsaved() {
            return;
        }

        let source = starter_source();
        self.source = source;
        self.saved_source.clear();
        self.current_path = self
            .project_root
            .as_ref()
            .map(|root| root.join("adsiz.ana").display().to_string())
            .unwrap_or_else(|| "adsiz.ana".to_string());
        self.rename_file_name = "adsiz.ana".to_string();
        self.output = "Yeni dosya olusturuldu.".to_string();
        self.build_output = "Henuz build yok.".to_string();
        self.status = "Yeni dosya".to_string();
        self.build_exe = None;
        self.save_ide_state();
        self.check_silent();
    }

    fn create_project_file(&mut self) {
        if !self.confirm_discard_unsaved() {
            return;
        }

        let Some(root) = self.project_root.clone() else {
            self.report_io_error("Yeni dosya icin once proje klasoru ac.");
            return;
        };

        let path = match project_child_path(&root, &self.new_file_name) {
            Ok(path) => path,
            Err(message) => {
                self.report_io_error(message);
                return;
            }
        };

        if path.exists() {
            self.report_io_error(format!("Dosya zaten var `{}`", path.display()));
            return;
        }

        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                self.report_io_error(format!(
                    "Klasor olusturulamadi `{}`: {error}",
                    parent.display()
                ));
                return;
            }
        }

        let source = starter_source();
        match fs::write(&path, source) {
            Ok(()) => {
                self.refresh_project_files();
                self.load_path(&path);
                self.status = "Dosya olusturuldu".to_string();
            }
            Err(error) => {
                self.report_io_error(format!(
                    "Dosya olusturulamadi `{}`: {error}",
                    path.display()
                ));
            }
        }
    }

    fn rename_current_file(&mut self) {
        if self.current_path_is_placeholder() {
            self.report_io_error("Yeniden adlandirmak icin once dosyayi kaydet.");
            return;
        }

        if self.is_dirty() && !self.save_current_path() {
            return;
        }

        let current = PathBuf::from(self.current_path.trim());
        let target = match sibling_file_path(&current, &self.rename_file_name) {
            Ok(path) => path,
            Err(message) => {
                self.report_io_error(message);
                return;
            }
        };

        if target == current {
            self.status = "Dosya adi degismedi".to_string();
            return;
        }

        if target.exists() {
            self.report_io_error(format!("Hedef dosya zaten var `{}`", target.display()));
            return;
        }

        match fs::rename(&current, &target) {
            Ok(()) => {
                self.current_path = target.display().to_string();
                self.rename_file_name = target
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("adsiz.ana")
                    .to_string();
                self.build_exe = None;
                self.build_output = "Henuz build yok.".to_string();
                self.refresh_project_files();
                self.save_ide_state();
                self.status = "Dosya yeniden adlandirildi".to_string();
                self.output = format!(
                    "Yeniden adlandirildi:\n{} -> {}",
                    current.display(),
                    target.display()
                );
            }
            Err(error) => {
                self.report_io_error(format!(
                    "Dosya yeniden adlandirilamadi `{}` -> `{}`: {error}",
                    current.display(),
                    target.display()
                ));
            }
        }
    }

    fn delete_current_file(&mut self) {
        if self.current_path_is_placeholder() {
            self.report_io_error("Silmek icin kayitli bir dosya sec.");
            return;
        }

        let path = PathBuf::from(self.current_path.trim());
        let confirmed = matches!(
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title("Dosya silinsin mi?")
                .set_description(format!("Bu dosya silinecek:\n{}", path.display()))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(),
            rfd::MessageDialogResult::Yes
        );

        if !confirmed {
            return;
        }

        match fs::remove_file(&path) {
            Ok(()) => {
                self.refresh_project_files();
                self.reset_to_new_buffer();
                self.save_ide_state();
                self.status = "Dosya silindi".to_string();
                self.output = format!("Silindi:\n{}", path.display());
            }
            Err(error) => {
                self.report_io_error(format!("Dosya silinemedi `{}`: {error}", path.display()));
            }
        }
    }

    fn reset_to_new_buffer(&mut self) {
        let source = starter_source();
        self.saved_source = source.clone();
        self.source = source;
        self.current_path = self
            .project_root
            .as_ref()
            .map(|root| root.join("adsiz.ana").display().to_string())
            .unwrap_or_else(|| "adsiz.ana".to_string());
        self.rename_file_name = "adsiz.ana".to_string();
        self.build_exe = None;
        self.build_output = "Henuz build yok.".to_string();
        self.save_ide_state();
        self.check_silent();
    }

    fn ensure_project_root_for_file(&mut self, path: &Path) {
        if self
            .project_root
            .as_ref()
            .is_some_and(|root| path.starts_with(root))
        {
            self.refresh_project_files();
            return;
        }

        if let Some(parent) = path.parent() {
            self.project_root = Some(parent.to_path_buf());
            self.refresh_project_files();
        }
    }

    fn save_ide_state(&self) {
        let current_path = if self.current_path_is_placeholder() {
            None
        } else {
            Some(Path::new(self.current_path.trim()))
        };

        write_ide_state(self.project_root.as_deref(), current_path);
    }

    fn report_io_error(&mut self, message: impl Into<String>) {
        self.diagnostics = vec![Diagnostic::io(message)];
        self.status = "Dosya islemi hatasi".to_string();
        self.selected_tab = Tab::Diagnostics;
    }

    fn confirm_discard_unsaved(&self) -> bool {
        if !self.is_dirty() {
            return true;
        }

        matches!(
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title("Kaydedilmemis degisiklik")
                .set_description("Kaydedilmemis degisiklikler var. Kaydetmeden devam edilsin mi?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(),
            rfd::MessageDialogResult::Yes
        )
    }

    fn handle_shortcuts(&mut self, context: &egui::Context) {
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            self.open_file_dialog();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            self.save_current_path();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.run_selected_mode();
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::B)) {
            self.run_mode = RunMode::Compile;
            self.build();
        }
        if context.input_mut(|input| {
            input.consume_key(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::F5,
            )
        }) {
            self.run_built_exe();
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

    fn project_root_label(&self) -> String {
        self.project_root
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Proje klasoru acilmadi.".to_string())
    }

    fn relative_project_path(&self, path: &Path) -> String {
        self.project_root
            .as_ref()
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn configure_fonts(context: &egui::Context) {
    let mut style = (*context.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(37, 37, 38);
    style.visuals.window_fill = Color32::from_rgb(30, 30, 30);
    style.visuals.extreme_bg_color = Color32::from_rgb(30, 30, 30);
    style.visuals.faint_bg_color = Color32::from_rgb(45, 45, 48);
    style.visuals.selection.bg_fill = Color32::from_rgb(38, 79, 120);
    style.visuals.hyperlink_color = Color32::from_rgb(86, 156, 214);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 48);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(62, 62, 66);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(38, 79, 120);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(37, 37, 38);
    style.visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(58, 58, 60));
    style.visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(86, 156, 214));
    style.visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(86, 156, 214));
    style.spacing.item_spacing = egui::vec2(6.0, 5.0);
    style.interaction.resize_grab_radius_side = 2.0;
    context.set_global_style(style);
}

fn panel_frame(fill: Color32, x_margin: i8, y_margin: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(x_margin, y_margin))
}

#[derive(Debug, Default, PartialEq, Eq)]
struct IdeSavedState {
    project_root: Option<PathBuf>,
    current_path: Option<PathBuf>,
}

fn load_ide_state() -> Option<IdeSavedState> {
    let path = ide_state_path()?;
    let source = fs::read_to_string(path).ok()?;
    Some(parse_ide_state(&source))
}

fn write_ide_state(project_root: Option<&Path>, current_path: Option<&Path>) {
    let Some(path) = ide_state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let _ = fs::write(path, format_ide_state(project_root, current_path));
}

fn ide_state_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(|path| PathBuf::from(path).join("Anadil").join("ide-state.txt"))
        .or_else(|| {
            env::current_dir()
                .ok()
                .map(|path| path.join(".anadil-ide-state"))
        })
}

fn parse_ide_state(source: &str) -> IdeSavedState {
    let mut state = IdeSavedState::default();

    for line in source.lines() {
        if let Some(value) = line.strip_prefix("project_root=") {
            state.project_root = non_empty_path(value);
        } else if let Some(value) = line.strip_prefix("current_path=") {
            state.current_path = non_empty_path(value);
        }
    }

    state
}

fn format_ide_state(project_root: Option<&Path>, current_path: Option<&Path>) -> String {
    format!(
        "project_root={}\ncurrent_path={}\n",
        project_root
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        current_path
            .map(|path| path.display().to_string())
            .unwrap_or_default()
    )
}

fn non_empty_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
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

fn project_child_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = safe_relative_path(name)?;
    Ok(root.join(relative))
}

fn safe_relative_path(name: &str) -> Result<PathBuf, String> {
    let name = ensure_ana_extension(name.trim());
    if name.is_empty() {
        return Err("Dosya adi bos olamaz.".to_string());
    }

    let path = PathBuf::from(name);
    if path.is_absolute() {
        return Err("Dosya adi proje klasoru icinde goreli olmalidir.".to_string());
    }

    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("Dosya adi proje klasoru disina cikamaz.".to_string());
            }
        }
    }

    if safe.as_os_str().is_empty() {
        Err("Dosya adi bos olamaz.".to_string())
    } else {
        Ok(safe)
    }
}

fn sibling_file_path(current: &Path, name: &str) -> Result<PathBuf, String> {
    let name = ensure_ana_extension(name.trim());
    if name.is_empty() {
        return Err("Yeni dosya adi bos olamaz.".to_string());
    }

    let path = Path::new(&name);
    if path.components().count() != 1 || path.file_name().is_none() {
        return Err("Yeniden adlandirma icin sadece dosya adi yaz.".to_string());
    }

    let parent = current
        .parent()
        .ok_or_else(|| "Aktif dosyanin klasoru bulunamadi.".to_string())?;
    Ok(parent.join(path))
}

fn ensure_ana_extension(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    let path = Path::new(name);
    if path.extension().is_some() {
        name.to_string()
    } else {
        format!("{name}.ana")
    }
}

fn editor_line_count(source: &str) -> usize {
    source.split('\n').count().max(1)
}

fn relative_component_depth(path: &str) -> usize {
    Path::new(path)
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count()
}

fn parent_hint(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|parent| parent.to_str())
        .filter(|parent| !parent.is_empty())
        .map(|parent| format!("({parent})"))
        .unwrap_or_default()
}

fn char_index_for_line_column(source: &str, line: usize, column: usize) -> usize {
    let target_line = line.max(1);
    let target_column = column.max(1);
    let mut char_index = 0;

    for (line_index, line_text) in source.split_inclusive('\n').enumerate() {
        let line_number = line_index + 1;
        let line_without_newline = line_text.trim_end_matches(['\r', '\n']);

        if line_number == target_line {
            let column_offset = target_column
                .saturating_sub(1)
                .min(line_without_newline.chars().count());
            return char_index + column_offset;
        }

        char_index += line_text.chars().count();
    }

    source.chars().count()
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

fn list_project_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_project_files(root, &mut files);
    files.sort_by_key(|path| path.display().to_string());
    files
}

fn collect_project_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if matches!(name, ".git" | "target") {
                continue;
            }
            collect_project_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "ana") {
            files.push(path);
        }
    }
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

fn run_executable(path: &Path) -> Result<std::process::Output, std::io::Error> {
    let mut command = Command::new(path);
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    command.output()
}

fn format_exe_run_output(path: &Path, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();

    format!(
        "Native executable calistirildi:\n{}\n\nExit: {}\n\nstdout:\n{}\n\nstderr:\n{}",
        path.display(),
        exit_code_label(&output.status),
        empty_label(stdout),
        empty_label(stderr),
    )
}

fn format_comparison_output(
    interpreter: &str,
    native: &str,
    stderr: &str,
    status: &std::process::ExitStatus,
) -> String {
    let result = if status.success() && interpreter == native {
        "AYNI"
    } else {
        "FARKLI"
    };

    format!(
        "Interpreter/native karsilastirma: {result}\n\nExit: {}\n\nInterpreter stdout:\n{}\n\nNative stdout:\n{}\n\nNative stderr:\n{}",
        exit_code_label(status),
        empty_label(interpreter),
        empty_label(native),
        empty_label(stderr),
    )
}

fn empty_label(text: &str) -> &str {
    if text.is_empty() {
        "(bos)"
    } else {
        text
    }
}

fn exit_code_label(status: &std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "process signal ile sonlandi".to_string())
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

#[cfg(test)]
mod tests {
    use super::{
        char_index_for_line_column, editor_line_count, format_ide_state, parent_hint,
        parse_ide_state, project_child_path, relative_component_depth, sibling_file_path,
    };
    use std::path::Path;

    #[test]
    fn project_child_path_adds_extension_and_stays_in_project() {
        let root = Path::new("proje");
        assert_eq!(
            project_child_path(root, "src/program").expect("path should be valid"),
            root.join("src").join("program.ana")
        );

        assert!(project_child_path(root, "../disari.ana").is_err());
        assert!(project_child_path(root, r"C:\disari.ana").is_err());
        assert!(project_child_path(root, "").is_err());
    }

    #[test]
    fn sibling_file_path_only_accepts_file_name() {
        let current = Path::new("proje").join("src").join("eski.ana");
        assert_eq!(
            sibling_file_path(&current, "yeni").expect("file name should be valid"),
            Path::new("proje").join("src").join("yeni.ana")
        );

        assert!(sibling_file_path(&current, "alt/yeni.ana").is_err());
        assert!(sibling_file_path(&current, "").is_err());
    }

    #[test]
    fn converts_diagnostic_line_column_to_char_index() {
        let source = "ilk\nüğç\nson";
        assert_eq!(char_index_for_line_column(source, 1, 1), 0);
        assert_eq!(char_index_for_line_column(source, 2, 2), 5);
        assert_eq!(
            char_index_for_line_column(source, 3, 99),
            source.chars().count()
        );
    }

    #[test]
    fn counts_editor_lines_like_a_text_editor() {
        assert_eq!(editor_line_count(""), 1);
        assert_eq!(editor_line_count("a\nb"), 2);
        assert_eq!(editor_line_count("a\n"), 2);
    }

    #[test]
    fn parses_and_formats_ide_state() {
        let root = Path::new(r"C:\projeler\anadil");
        let file = root.join("main.ana");
        let source = format_ide_state(Some(root), Some(&file));
        let parsed = parse_ide_state(&source);

        assert_eq!(parsed.project_root.as_deref(), Some(root));
        assert_eq!(parsed.current_path.as_deref(), Some(file.as_path()));
    }

    #[test]
    fn describes_relative_file_depth() {
        assert_eq!(relative_component_depth("main.ana"), 1);
        assert_eq!(relative_component_depth(r"src\main.ana"), 2);
        assert_eq!(parent_hint(r"src\main.ana"), "(src)");
        assert_eq!(parent_hint("main.ana"), "");
    }
}
