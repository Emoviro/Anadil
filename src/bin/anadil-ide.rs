use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// IDE'den spawn edilen alt process'lerin Windows uzerinde kendi console
/// penceresini acmasini engeller. IDE GUI subsystem oldugu icin console
/// subsystem alt process'ler (anadil.exe, uretilen .exe) bu flag olmadan
/// gecici bir cmd penceresi flash eder.
fn hide_command_window(command: &mut Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = command;
    }
}

use anadil::{check_source, diagnostics::Diagnostic};
use eframe::egui::{
    self, text::LayoutJob, text_edit::TextEditState, Color32, FontFamily, FontId, RichText,
    ScrollArea, TextEdit, TextFormat,
};

// ============================================================
// Anadil Tema — "Bakir Gece"
// Modern dark zemin + sicak bakir/amber aksan.
// ============================================================

// Backgrounds (deepest -> highest)
const BG_BASE: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x26);
const BG_PANEL: Color32 = Color32::from_rgb(0x21, 0x23, 0x30);
const BG_EDITOR: Color32 = Color32::from_rgb(0x16, 0x17, 0x21);
const BG_RAISED: Color32 = Color32::from_rgb(0x2A, 0x2C, 0x3D);
const BG_RAISED_HI: Color32 = Color32::from_rgb(0x36, 0x39, 0x50);
const BG_INPUT: Color32 = Color32::from_rgb(0x14, 0x15, 0x1D);

// Borders
const BORDER: Color32 = Color32::from_rgb(0x2D, 0x30, 0x44);
const BORDER_STRONG: Color32 = Color32::from_rgb(0x40, 0x43, 0x5C);

// Text
const FG_PRIMARY: Color32 = Color32::from_rgb(0xE8, 0xE9, 0xF2);
const FG_SECONDARY: Color32 = Color32::from_rgb(0xA4, 0xA6, 0xB8);
const FG_TERTIARY: Color32 = Color32::from_rgb(0x70, 0x73, 0x8A);

// Accent — sicak bakir (Anadil imzasi)
const ACCENT: Color32 = Color32::from_rgb(0xE8, 0xA8, 0x57);
const ACCENT_HOVER: Color32 = Color32::from_rgb(0xF4, 0xBE, 0x73);
const ACCENT_DIM: Color32 = Color32::from_rgb(0x4F, 0x39, 0x1D);
const ACCENT_GLOW: Color32 = Color32::from_rgb(0x6B, 0x4D, 0x27);

// Selection
const SELECTION_BG: Color32 = Color32::from_rgb(0x3B, 0x3F, 0x6B);
const HYPERLINK: Color32 = Color32::from_rgb(0x82, 0xAA, 0xFF);

// Status
const STATUS_OK: Color32 = Color32::from_rgb(0x88, 0xC9, 0x7A);
const STATUS_ERROR: Color32 = Color32::from_rgb(0xFF, 0x8B, 0x92);
const STATUS_WARN: Color32 = Color32::from_rgb(0xFF, 0xC9, 0x87);

// Editor error line background
const ERR_LINE_BG: Color32 = Color32::from_rgb(0x33, 0x1B, 0x24);

// Syntax highlighting (palet uyumlu)
const SYN_KEYWORD: Color32 = ACCENT; // egil/dön/döngü/...
const SYN_TYPE: Color32 = Color32::from_rgb(0x7F, 0xCB, 0xC4); // sayı/mantık/metin
const SYN_BUILTIN: Color32 = Color32::from_rgb(0x82, 0xAA, 0xFF); // Ana/yazdır/uzunluk
const SYN_FUNCTION: Color32 = Color32::from_rgb(0xFF, 0xD4, 0x66); // Buyuk harfli ident
const SYN_STRING: Color32 = Color32::from_rgb(0xC3, 0xE8, 0x8D);
const SYN_NUMBER: Color32 = Color32::from_rgb(0xF7, 0x8C, 0x6C);
const SYN_COMMENT: Color32 = Color32::from_rgb(0x60, 0x63, 0x7E);
const SYN_PUNCT: Color32 = Color32::from_rgb(0xC5, 0xC6, 0xD6);

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
    new_file_name: String,
    rename_file_name: String,
    selected_diagnostic: Option<usize>,
    pending_editor_jump: Option<(usize, usize)>,
    left_panel_width: f32,
    bottom_panel_height: f32,
    build_tools_available: bool,
}

const LEFT_PANEL_MIN: f32 = 220.0;
const LEFT_PANEL_MAX: f32 = 520.0;
const BOTTOM_PANEL_MIN: f32 = 140.0;
const BOTTOM_PANEL_MAX: f32 = 480.0;

impl Default for AnadilIde {
    fn default() -> Self {
        let source = starter_source();
        Self {
            saved_source: source.clone(),
            source,
            current_path: "yeni.ana".to_string(),
            project_root: None,
            project_files: Vec::new(),
            status: "Hazir".to_string(),
            output: "Henüz calistirma yok.".to_string(),
            build_output: "Henuz build yok.".to_string(),
            diagnostics: Vec::new(),
            examples: list_examples(),
            selected_tab: Tab::Output,
            build_exe: None,
            new_file_name: "yeni.ana".to_string(),
            rename_file_name: "yeni.ana".to_string(),
            selected_diagnostic: None,
            pending_editor_jump: None,
            left_panel_width: 290.0,
            bottom_panel_height: 220.0,
            build_tools_available: detect_build_tools(),
        }
    }
}

impl eframe::App for AnadilIde {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        self.handle_shortcuts(&context);
        context.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        // Dis CentralPanel: bare ui arka plansiz; tum alani BG_EDITOR ile boyar.
        let outer_frame = egui::Frame::new().fill(BG_EDITOR);
        egui::CentralPanel::default()
            .frame(outer_frame)
            .show_inside(ui, |ui| {
                egui::Panel::top("anadil_top_v3")
                    .exact_size(52.0)
                    .frame(panel_frame(BG_PANEL, 14, 6))
                    .show_inside(ui, |ui| self.top_bar(ui));

                self.bottom_panel_height = self
                    .bottom_panel_height
                    .clamp(BOTTOM_PANEL_MIN, BOTTOM_PANEL_MAX);
                let bottom_panel = egui::Panel::bottom("anadil_bottom_v3")
                    .resizable(true)
                    .show_separator_line(true)
                    .default_size(self.bottom_panel_height)
                    .min_size(BOTTOM_PANEL_MIN)
                    .max_size(BOTTOM_PANEL_MAX)
                    .frame(panel_frame(BG_EDITOR, 14, 8))
                    .show_inside(ui, |ui| self.bottom_panel(ui));
                self.bottom_panel_height = bottom_panel
                    .response
                    .rect
                    .height()
                    .clamp(BOTTOM_PANEL_MIN, BOTTOM_PANEL_MAX);

                egui::CentralPanel::default()
                    .frame(panel_frame(BG_EDITOR, 14, 10))
                    .show_inside(ui, |ui| self.workspace_panel(ui));
            });
    }
}

impl AnadilIde {
    fn workspace_panel(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let grip_width = 6.0;
        let min_editor_width = 320.0;
        let max_left =
            LEFT_PANEL_MAX.min((available.x - grip_width - min_editor_width).max(LEFT_PANEL_MIN));
        self.left_panel_width = self.left_panel_width.clamp(LEFT_PANEL_MIN, max_left);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            ui.allocate_ui_with_layout(
                egui::vec2(self.left_panel_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(BG_PANEL)
                        .inner_margin(egui::Margin::symmetric(12, 12))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());
                            self.left_panel(ui);
                        });
                },
            );

            self.draw_left_resize_handle(ui, available.y, max_left);

            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(BG_EDITOR)
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());
                            self.editor_panel(ui);
                        });
                },
            );
        });
    }

    fn draw_left_resize_handle(&mut self, ui: &mut egui::Ui, height: f32, max_left: f32) {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(6.0, height), egui::Sense::drag());
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        if response.dragged() {
            self.left_panel_width =
                (self.left_panel_width + response.drag_delta().x).clamp(LEFT_PANEL_MIN, max_left);
        }

        let (fill, stroke) = if response.hovered() || response.dragged() {
            (BG_RAISED, egui::Stroke::new(1.0, ACCENT))
        } else {
            (BG_EDITOR, egui::Stroke::new(1.0, BORDER))
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, fill);
        ui.painter().vline(rect.center().x, rect.y_range(), stroke);
    }

    fn new() -> Self {
        let mut ide = Self::default();
        ide.restore_last_session();
        ide
    }

    fn restore_last_session(&mut self) {
        let Some(state) = load_ide_state() else {
            return;
        };

        self.apply_saved_state(state);
    }

    fn apply_saved_state(&mut self, state: IdeSavedState) {
        if let Some(root) = state.project_root.filter(|path| path.is_dir()) {
            self.project_root = Some(root);
            self.refresh_project_files();
        }

        if let Some(path) = state.current_path.filter(|path| path.is_file()) {
            self.load_path(&path);
        }

        if let Some(width) = state.left_panel_width {
            self.left_panel_width = width.clamp(LEFT_PANEL_MIN, LEFT_PANEL_MAX);
        }
        if let Some(height) = state.bottom_panel_height {
            self.bottom_panel_height = height.clamp(BOTTOM_PANEL_MIN, BOTTOM_PANEL_MAX);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.add_space(2.0);
            ui.label(RichText::new("◆").size(15.0).color(ACCENT));
            ui.add_space(2.0);
            ui.label(RichText::new("Anadil").strong().size(19.0).color(ACCENT));
            ui.label(RichText::new("IDE").size(13.0).color(FG_TERTIARY));

            ui.add_space(10.0);
            vertical_divider(ui);
            ui.add_space(6.0);

            if ui.button("Aç").on_hover_text("Ctrl+O").clicked() {
                self.open_file_dialog();
            }
            if ui.button("Kaydet").on_hover_text("Ctrl+S").clicked() {
                self.save_current_path();
            }
            if ui.button("Farklı Kaydet").clicked() {
                self.save_as_dialog();
            }
            if ui.button("Klasör Aç").clicked() {
                self.open_folder_dialog();
            }

            ui.add_space(6.0);
            vertical_divider(ui);
            ui.add_space(6.0);

            if ui.button("Kontrol").clicked() {
                self.check();
            }

            let run_button = egui::Button::new(RichText::new("▶  Yap").color(BG_BASE).strong())
                .fill(ACCENT)
                .corner_radius(egui::CornerRadius::same(6));
            let run_response = ui.add_enabled(self.build_tools_available, run_button);
            let run_response = if self.build_tools_available {
                run_response.on_hover_text("F5")
            } else {
                run_response.on_hover_text("Visual Studio Build Tools gerekli")
            };
            if run_response.clicked() {
                self.build_and_run();
            }

            let build_response =
                ui.add_enabled(self.build_tools_available, egui::Button::new("EXE Derle"));
            let build_response = if self.build_tools_available {
                build_response.on_hover_text("Ctrl+B")
            } else {
                build_response.on_hover_text("Visual Studio Build Tools gerekli")
            };
            if build_response.clicked() {
                self.build();
            }
            if ui
                .add_enabled(self.build_exe.is_some(), egui::Button::new("EXE Çalıştır"))
                .on_hover_text("Ctrl+Shift+F5")
                .clicked()
            {
                self.run_built_exe();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (dot_color, label_color, label_text) = if self.is_dirty() {
                    (STATUS_WARN, STATUS_WARN, "Değişiklik var")
                } else {
                    (STATUS_OK, FG_SECONDARY, "Kayıtlı")
                };
                ui.label(RichText::new(label_text).small().color(label_color));
                ui.label(RichText::new("●").size(10.0).color(dot_color));
                ui.add_space(8.0);
                if !self.build_tools_available {
                    ui.label(RichText::new("Build Tools yok").small().color(STATUS_WARN));
                    ui.add_space(8.0);
                }
                ui.label(RichText::new(&self.status).small().color(FG_SECONDARY));
            });
        });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        // Frame'in panel_rect'i tam doldurmasi icin (resize'in PanelState'e dogru kaydedilmesi sart)
        ui.take_available_width();

        section_header(ui, "PROJE");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui
                .button("Klasör")
                .on_hover_text("Proje klasörü aç")
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

        ui.add_space(8.0);
        ui.label(
            RichText::new(self.project_root_label())
                .small()
                .color(FG_TERTIARY),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut self.new_file_name)
                    .desired_width(f32::INFINITY)
                    .hint_text("yeni.ana"),
            );
            if ui
                .add_enabled(self.project_root.is_some(), egui::Button::new("Oluştur"))
                .clicked()
            {
                self.create_project_file();
            }
        });

        ui.add_space(12.0);
        horizontal_divider(ui);
        ui.add_space(8.0);
        section_header(ui, "DOSYALAR");
        ui.add_space(4.0);

        let file_list_height = (ui.available_height() - 200.0).max(180.0);
        ScrollArea::vertical()
            .id_salt("project_files")
            .max_height(file_list_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.project_root.is_none() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Bir proje klasörü aç.")
                            .small()
                            .color(FG_TERTIARY),
                    );
                } else if self.project_files.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Bu klasörde .ana dosyası yok.")
                            .small()
                            .color(FG_TERTIARY),
                    );
                }

                let files = self.project_files.clone();
                for path in files {
                    self.project_file_row(ui, &path);
                }
            });

        ui.add_space(10.0);
        horizontal_divider(ui);
        ui.add_space(6.0);

        egui::CollapsingHeader::new(RichText::new("Aktif Dosya").color(FG_SECONDARY).strong())
            .default_open(false)
            .show(ui, |ui| {
                ui.label(RichText::new("Aktif yol").small().color(FG_TERTIARY));
                ui.add(
                    TextEdit::singleline(&mut self.current_path)
                        .desired_width(f32::INFINITY)
                        .hint_text("examples\\topla.ana"),
                );
                if ui.button("Bu yoldan aç").clicked() {
                    self.open_current_path();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Yeni ad").small().color(FG_TERTIARY));
                ui.add(
                    TextEdit::singleline(&mut self.rename_file_name)
                        .desired_width(f32::INFINITY)
                        .hint_text("ornek.ana"),
                );
                ui.horizontal(|ui| {
                    let has_real_file = !self.current_path_is_placeholder();
                    if ui
                        .add_enabled(has_real_file, egui::Button::new("Yeniden Adlandır"))
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

        egui::CollapsingHeader::new(RichText::new("Örnekler").color(FG_SECONDARY).strong())
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
                        let text =
                            RichText::new(name).color(if selected { ACCENT } else { FG_PRIMARY });
                        if ui.selectable_label(selected, text).clicked() {
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
            ui.add_space(depth as f32 * 14.0);
            if selected {
                ui.label(RichText::new("▸").color(ACCENT));
            } else {
                ui.add_space(10.0);
            }
            let color = if selected { ACCENT } else { FG_PRIMARY };

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
            if self.is_dirty() {
                ui.label(RichText::new("●").size(14.0).color(ACCENT))
                    .on_hover_text("Kaydedilmemiş değişiklik var");
            } else {
                ui.label(RichText::new("●").size(14.0).color(FG_TERTIARY));
            }
            ui.label(
                RichText::new(self.display_path())
                    .strong()
                    .size(13.5)
                    .color(FG_PRIMARY),
            );
            if let Some(exe) = &self.build_exe {
                ui.add_space(6.0);
                ui.label(RichText::new("│").color(BORDER_STRONG));
                ui.add_space(6.0);
                ui.label(RichText::new("✔").color(STATUS_OK).size(13.0));
                ui.label(
                    RichText::new(format!("EXE: {exe}"))
                        .small()
                        .color(STATUS_OK),
                );
            }
        });

        ui.add_space(8.0);
        self.apply_pending_editor_jump(ui);

        let diagnostics = self.diagnostics.clone();
        let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = highlight_job(text.as_str(), &diagnostics);
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        let editor_id = ui.make_persistent_id("source_editor");
        let line_count = editor_line_count(&self.source);
        let previous_source = self.source.clone();
        let editor_height = ui.available_height();
        let min_editor_height = editor_height.max(line_count.max(26) as f32 * 20.0);
        let editor_width = ui.available_width();
        let output = ScrollArea::vertical()
            .id_salt("source_editor_scroll")
            .auto_shrink([false, false])
            .max_height(editor_height)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    line_number_gutter(ui, line_count, min_editor_height);
                    ui.add_space(10.0);
                    TextEdit::multiline(&mut self.source)
                        .id(editor_id)
                        .font(FontId::new(15.0, FontFamily::Monospace))
                        .desired_width(f32::INFINITY)
                        .desired_rows(line_count.max(26))
                        .min_size(egui::vec2(
                            (editor_width - line_number_gutter_width(line_count) - 10.0).max(320.0),
                            min_editor_height,
                        ))
                        .lock_focus(true)
                        .layouter(&mut layouter)
                        .show(ui)
                })
                .inner
            })
            .inner;

        if output.response.changed() {
            if let Some(cursor_range) = output.cursor_range {
                let cursor_index = cursor_range.primary.index;
                if let Some(new_cursor) =
                    apply_editor_smart_edit(&previous_source, &mut self.source, cursor_index)
                {
                    let mut state = output.state;
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor),
                        )));
                    state.store(ui.ctx(), output.response.id);
                    ui.ctx()
                        .memory_mut(|memory| memory.request_focus(output.response.id));
                }
            }
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

    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        // Frame'in panel_rect'i tam doldurmasi icin (resize'in PanelState'e dogru kaydedilmesi sart)
        ui.take_available_height();

        ui.horizontal(|ui| {
            self.tab_button(ui, Tab::Output, "Çıktı");
            ui.add_space(4.0);
            self.tab_button(ui, Tab::Diagnostics, "Tanılama");
            ui.add_space(4.0);
            self.tab_button(ui, Tab::Build, "Build");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let n = self.diagnostics.len();
                let (text, color) = if n == 0 {
                    ("0 tanılama".to_string(), FG_TERTIARY)
                } else {
                    (format!("{n} tanılama"), STATUS_ERROR)
                };
                ui.label(RichText::new(text).small().color(color));
            });
        });
        ui.add_space(2.0);
        horizontal_divider(ui);
        ui.add_space(8.0);

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

    fn tab_button(&mut self, ui: &mut egui::Ui, tab: Tab, label: &str) {
        let active = self.selected_tab == tab;
        let font = FontId::proportional(13.0);
        let padding = egui::vec2(10.0, 6.0);

        // Olcumu icin gecici layout
        let measure = ui
            .painter()
            .layout_no_wrap(label.to_string(), font.clone(), FG_PRIMARY);
        let desired = egui::vec2(
            measure.size().x + padding.x * 2.0,
            measure.size().y + padding.y * 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

        let hovered = response.hovered();
        let label_color = if active {
            ACCENT
        } else if hovered {
            ACCENT_HOVER
        } else {
            FG_SECONDARY
        };

        if active || hovered {
            let fill = if active {
                BG_RAISED
            } else {
                BG_RAISED.gamma_multiply(0.6)
            };
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(4), fill);
        }

        let galley = ui
            .painter()
            .layout_no_wrap(label.to_string(), font, label_color);
        ui.painter()
            .galley(rect.left_top() + padding, galley, label_color);

        if active {
            let underline_y = rect.bottom() - 1.0;
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 6.0, underline_y),
                    egui::pos2(rect.right() - 6.0, underline_y),
                ],
                egui::Stroke::new(2.0, ACCENT),
            );
        }

        if response.clicked() {
            self.selected_tab = tab;
        }
    }

    fn diagnostics_ui(&mut self, ui: &mut egui::Ui) {
        if self.diagnostics.is_empty() {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("✓").size(22.0).color(STATUS_OK));
                ui.add_space(4.0);
                ui.label(RichText::new("Tanılama temiz — hata yok.").color(FG_SECONDARY));
            });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            let diagnostics = self.diagnostics.clone();
            for (index, diagnostic) in diagnostics.iter().enumerate() {
                let place = diagnostic
                    .span
                    .map(|span| format!("satır {}, sütun {}", span.line, span.column))
                    .unwrap_or_else(|| "konum belirtilmemiş".to_string());

                let active = self.selected_diagnostic == Some(index);
                let fill = if active { BG_RAISED_HI } else { BG_RAISED };
                let stroke = if active {
                    egui::Stroke::new(1.0, ACCENT)
                } else {
                    egui::Stroke::new(1.0, BORDER)
                };
                let stripe = match diagnostic.severity.as_str() {
                    "warning" => STATUS_WARN,
                    "info" => HYPERLINK,
                    _ => STATUS_ERROR,
                };

                let frame = egui::Frame::new()
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 10));

                let card = frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Sol kenarda severity stripe
                        let (_id, stripe_rect) = ui.allocate_space(egui::vec2(3.0, 38.0));
                        ui.painter()
                            .rect_filled(stripe_rect, egui::CornerRadius::same(2), stripe);
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(diagnostic.severity.as_str().to_uppercase())
                                        .strong()
                                        .small()
                                        .color(stripe),
                                );
                                ui.label(RichText::new("·").color(FG_TERTIARY));
                                ui.label(
                                    RichText::new(diagnostic.stage.as_str())
                                        .small()
                                        .color(FG_SECONDARY),
                                );
                                ui.add_space(6.0);
                                ui.label(RichText::new(place).small().color(FG_TERTIARY));
                            });
                            ui.add_space(2.0);
                            ui.label(RichText::new(&diagnostic.message).color(FG_PRIMARY));
                            if diagnostic.span.is_some() {
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new("→ tıkla: editöre git").small().color(ACCENT),
                                );
                            }
                        });
                    });
                });

                let response = card.response.interact(egui::Sense::click());
                if diagnostic.span.is_some() {
                    response.clone().on_hover_text("Editor konumuna git");
                }

                if response.clicked() {
                    self.jump_to_diagnostic(index);
                }
                ui.add_space(6.0);
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

    fn build(&mut self) {
        if !self.ensure_build_tools_available_for_action() {
            return;
        }
        let _ = self.build_native();
    }

    fn build_and_run(&mut self) {
        if !self.ensure_build_tools_available_for_action() {
            return;
        }
        if self.build_native() {
            self.run_built_exe();
        }
    }

    fn ensure_build_tools_available_for_action(&mut self) -> bool {
        if self.build_tools_available {
            true
        } else {
            self.report_build_tools_missing();
            false
        }
    }

    fn report_build_tools_missing(&mut self) {
        self.status = "Build Tools gerekli".to_string();
        self.build_output = "Visual Studio Build Tools gerekli\n\nEXE Derle ve Yap/F5 native executable uretir. Native derleme icin Visual Studio Build Tools kurulmalidir.".to_string();
        self.selected_tab = Tab::Build;
    }

    fn build_native(&mut self) -> bool {
        self.check_silent();
        if !self.diagnostics.is_empty() {
            self.build_exe = None;
            self.status = "Build once compile hatasini duzeltmeli".to_string();
            self.selected_tab = Tab::Diagnostics;
            return false;
        }

        let Some(path) = self.prepare_build_source_path() else {
            self.build_exe = None;
            return false;
        };

        self.build_output = format_build_started(&path);
        self.selected_tab = Tab::Build;

        match run_native_build(&path) {
            Ok(build) => {
                self.build_exe = Some(build.exe.clone());
                self.build_output = format_build_success(&path, &build);
                self.status = "EXE derlendi".to_string();
                self.selected_tab = Tab::Build;
                true
            }
            Err(message) => {
                self.build_exe = None;
                self.build_output = message.clone();
                self.diagnostics = vec![Diagnostic::native(message)];
                self.status = "Native build hatasi".to_string();
                self.selected_tab = Tab::Diagnostics;
                false
            }
        }
    }

    fn run_built_exe(&mut self) {
        let Some(exe) = self.build_exe.clone() else {
            self.build_output = format_no_exe_to_run();
            self.status = "Calistirilacak EXE yok".to_string();
            self.selected_tab = Tab::Build;
            return;
        };

        let exe_path = PathBuf::from(&exe);
        match run_executable(&exe_path) {
            Ok(output) => {
                self.output = format_program_output(&output);
                self.build_output = format_exe_run_output(&exe_path, &output);
                if output.status.success() {
                    self.status = "EXE calistirildi".to_string();
                    self.diagnostics.clear();
                    self.selected_tab = Tab::Output;
                } else {
                    let code = exit_code_label(&output.status);
                    self.status = "EXE hata ile bitti".to_string();
                    self.diagnostics = vec![Diagnostic::native(format!(
                        "Native executable basarisiz bitti: {code}"
                    ))];
                    self.selected_tab = Tab::Build;
                }
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

    fn prepare_build_source_path(&mut self) -> Option<PathBuf> {
        if self.current_path_is_placeholder() || self.is_dirty() {
            self.status = "Build icin once kaydediliyor".to_string();
            self.build_output =
                "Build hazirligi\n\nKaynak dosya kayitli degil veya degisti. Build oncesi kaydediliyor.".to_string();
            self.selected_tab = Tab::Build;
            if !self.save_current_path() {
                self.status = "Build iptal edildi".to_string();
                self.build_output =
                    "Build iptal edildi\n\nKaynak dosya kaydedilemedi veya kaydetme islemi iptal edildi.\nDerleme icin once `.ana` dosyasini kaydetmelisin.".to_string();
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
            .map(|root| root.join("yeni.ana").display().to_string())
            .unwrap_or_else(|| "yeni.ana".to_string());
        self.rename_file_name = "yeni.ana".to_string();
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

        write_ide_state(
            self.project_root.as_deref(),
            current_path,
            Some(self.left_panel_width),
            Some(self.bottom_panel_height),
        );
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
            if self.build_tools_available {
                self.build_and_run();
            } else {
                self.report_build_tools_missing();
            }
        }
        if context.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::B)) {
            if self.build_tools_available {
                self.build();
            } else {
                self.report_build_tools_missing();
            }
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

    // Backgrounds
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.window_fill = BG_EDITOR;
    style.visuals.extreme_bg_color = BG_INPUT;
    style.visuals.faint_bg_color = BG_RAISED;
    style.visuals.code_bg_color = BG_EDITOR;
    style.visuals.window_stroke = egui::Stroke::new(1.0, BORDER);

    // Selection / hyperlink
    style.visuals.selection.bg_fill = SELECTION_BG;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT_HOVER);
    style.visuals.hyperlink_color = HYPERLINK;

    // Default metin rengi
    style.visuals.override_text_color = Some(FG_PRIMARY);

    // Noninteractive (label, separator)
    style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    style.visuals.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, FG_SECONDARY);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);

    // Inactive (durmus dugmeler)
    style.visuals.widgets.inactive.bg_fill = BG_RAISED;
    style.visuals.widgets.inactive.weak_bg_fill = BG_RAISED;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, FG_PRIMARY);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.inactive.expansion = 0.0;

    // Hovered
    style.visuals.widgets.hovered.bg_fill = BG_RAISED_HI;
    style.visuals.widgets.hovered.weak_bg_fill = BG_RAISED_HI;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT_GLOW);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, ACCENT_HOVER);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.hovered.expansion = 1.0;

    // Active (basili / secili)
    style.visuals.widgets.active.bg_fill = ACCENT_DIM;
    style.visuals.widgets.active.weak_bg_fill = ACCENT_DIM;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.active.expansion = 0.0;

    // Open (acik combobox vb.)
    style.visuals.widgets.open.bg_fill = BG_RAISED_HI;
    style.visuals.widgets.open.weak_bg_fill = BG_RAISED_HI;
    style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, FG_PRIMARY);
    style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

    // Yuvarlatma
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(8);

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.indent = 14.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.icon_width = 16.0;
    style.spacing.icon_spacing = 6.0;
    style.spacing.menu_margin = egui::Margin::same(8);

    style.interaction.resize_grab_radius_side = 4.0;
    style.interaction.resize_grab_radius_corner = 6.0;

    context.set_global_style(style);
}

fn panel_frame(fill: Color32, x_margin: i8, y_margin: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(x_margin, y_margin))
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).strong().size(11.0).color(FG_TERTIARY));
}

fn vertical_divider(ui: &mut egui::Ui) {
    let height = 22.0;
    let (_id, rect) = ui.allocate_space(egui::vec2(1.0, height));
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        egui::Stroke::new(1.0, BORDER_STRONG),
    );
}

fn line_number_gutter(ui: &mut egui::Ui, line_count: usize, min_height: f32) {
    let width = line_number_gutter_width(line_count);
    ui.allocate_ui_with_layout(
        egui::vec2(width, min_height),
        egui::Layout::top_down(egui::Align::Max),
        |ui| {
            ui.set_width(width);
            let font = FontId::new(15.0, FontFamily::Monospace);
            for number in 1..=line_count {
                ui.label(
                    RichText::new(number.to_string())
                        .font(font.clone())
                        .color(FG_TERTIARY),
                );
            }
        },
    );
}

fn line_number_gutter_width(line_count: usize) -> f32 {
    let digits = line_count.max(1).to_string().len() as f32;
    (digits * 9.0 + 14.0).max(28.0)
}

fn horizontal_divider(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (_id, rect) = ui.allocate_space(egui::vec2(width, 1.0));
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, BORDER),
    );
}

#[derive(Debug, Default, PartialEq)]
struct IdeSavedState {
    project_root: Option<PathBuf>,
    current_path: Option<PathBuf>,
    left_panel_width: Option<f32>,
    bottom_panel_height: Option<f32>,
}

fn load_ide_state() -> Option<IdeSavedState> {
    let path = ide_state_path()?;
    let source = fs::read_to_string(path).ok()?;
    Some(parse_ide_state(&source))
}

fn write_ide_state(
    project_root: Option<&Path>,
    current_path: Option<&Path>,
    left_panel_width: Option<f32>,
    bottom_panel_height: Option<f32>,
) {
    let Some(path) = ide_state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let _ = fs::write(
        path,
        format_ide_state(
            project_root,
            current_path,
            left_panel_width,
            bottom_panel_height,
        ),
    );
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
        } else if let Some(value) = line.strip_prefix("left_panel_width=") {
            state.left_panel_width = value.trim().parse::<f32>().ok();
        } else if let Some(value) = line.strip_prefix("bottom_panel_height=") {
            state.bottom_panel_height = value.trim().parse::<f32>().ok();
        }
    }

    state
}

fn format_ide_state(
    project_root: Option<&Path>,
    current_path: Option<&Path>,
    left_panel_width: Option<f32>,
    bottom_panel_height: Option<f32>,
) -> String {
    format!(
        "project_root={}\ncurrent_path={}\nleft_panel_width={}\nbottom_panel_height={}\n",
        project_root
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        current_path
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        left_panel_width
            .map(|value| value.to_string())
            .unwrap_or_default(),
        bottom_panel_height
            .map(|value| value.to_string())
            .unwrap_or_default(),
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

fn apply_editor_smart_edit(
    previous_source: &str,
    current_source: &mut String,
    cursor_index: usize,
) -> Option<usize> {
    let (_start, inserted) = inserted_text(previous_source, current_source)?;

    if inserted == "\n" {
        return apply_auto_indent(current_source, cursor_index);
    }

    if inserted.chars().count() != 1 {
        return None;
    }

    let closing = match inserted.chars().next()? {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '"' => '"',
        '\'' => '\'',
        _ => return None,
    };

    insert_char_at_char_index(current_source, cursor_index, closing);
    Some(cursor_index)
}

fn apply_auto_indent(source: &mut String, cursor_index: usize) -> Option<usize> {
    if cursor_index == 0 {
        return None;
    }

    let before_cursor = slice_chars(source, 0, cursor_index);
    if !before_cursor.ends_with('\n') {
        return None;
    }

    let before_newline = before_cursor.trim_end_matches(['\r', '\n']);
    let previous_line = before_newline.rsplit('\n').next().unwrap_or("");
    let base_indent = leading_whitespace(previous_line);
    let extra_indent = previous_line.trim_end().ends_with('{');
    let next_char = source.chars().nth(cursor_index);

    let mut insertion = base_indent.to_string();
    if extra_indent {
        insertion.push_str("    ");
    }

    let cursor_after_indent = cursor_index + insertion.chars().count();

    if extra_indent && next_char == Some('}') {
        insertion.push('\n');
        insertion.push_str(base_indent);
    }

    if insertion.is_empty() {
        return None;
    }

    insert_str_at_char_index(source, cursor_index, &insertion);
    Some(cursor_after_indent)
}

fn inserted_text(previous: &str, current: &str) -> Option<(usize, String)> {
    let previous_chars = previous.chars().collect::<Vec<_>>();
    let current_chars = current.chars().collect::<Vec<_>>();

    if current_chars.len() < previous_chars.len() {
        return None;
    }

    let mut prefix = 0;
    while prefix < previous_chars.len()
        && prefix < current_chars.len()
        && previous_chars[prefix] == current_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < previous_chars.len().saturating_sub(prefix)
        && suffix < current_chars.len().saturating_sub(prefix)
        && previous_chars[previous_chars.len() - 1 - suffix]
            == current_chars[current_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    if previous_chars.len() - prefix - suffix != 0 {
        return None;
    }

    Some((
        prefix,
        current_chars[prefix..current_chars.len() - suffix]
            .iter()
            .collect(),
    ))
}

fn leading_whitespace(line: &str) -> &str {
    let end = line
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(line.len());
    &line[..end]
}

fn slice_chars(source: &str, start: usize, end: usize) -> String {
    source
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn insert_char_at_char_index(source: &mut String, char_index: usize, ch: char) {
    let mut buffer = [0; 4];
    insert_str_at_char_index(source, char_index, ch.encode_utf8(&mut buffer));
}

fn insert_str_at_char_index(source: &mut String, char_index: usize, text: &str) {
    let byte_index = byte_index_for_char(source, char_index);
    source.insert_str(byte_index, text);
}

fn byte_index_for_char(source: &str, char_index: usize) -> usize {
    source
        .char_indices()
        .map(|(index, _)| index)
        .nth(char_index)
        .unwrap_or(source.len())
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

fn detect_build_tools() -> bool {
    (command_available("ml64") && command_available("link") && command_available("lib"))
        || find_vcvars64().is_some()
}

fn command_available(command: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths).any(|dir| {
                command_candidates(command)
                    .iter()
                    .any(|candidate| dir.join(candidate).is_file())
            })
        })
        .unwrap_or(false)
}

fn command_candidates(command: &str) -> Vec<String> {
    if Path::new(command).extension().is_some() {
        return vec![command.to_string()];
    }

    vec![
        command.to_string(),
        format!("{command}.exe"),
        format!("{command}.cmd"),
        format!("{command}.bat"),
    ]
}

fn find_vcvars64() -> Option<PathBuf> {
    [
        r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|path| {
        path.is_file()
            && path
                .parent()
                .is_some_and(|parent| parent.join("vcvarsall.bat").is_file())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBuildOutput {
    exe: String,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBuildCommand {
    program: PathBuf,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
    display: String,
}

fn run_native_build(path: &Path) -> Result<NativeBuildOutput, String> {
    let build_command = native_build_command(path);
    let mut command = Command::new(&build_command.program);
    command.args(&build_command.args);
    if let Some(current_dir) = &build_command.current_dir {
        command.current_dir(current_dir);
    }
    hide_command_window(&mut command);

    let output = command
        .output()
        .map_err(|error| {
            format!(
                "Native build baslatilamadi\n\nKomut:\n{}\n\nHata:\n{error}\n\nNe yapmali:\nAnadil paketinde anadil.exe dosyasi var mi ve calistirilabiliyor mu kontrol et.",
                build_command.display
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let message = extract_json_string(&stdout, "message")
            .or_else(|| non_empty_trimmed(&stderr))
            .or_else(|| non_empty_trimmed(&stdout))
            .unwrap_or_else(|| {
                format!(
                    "Build process basarisiz bitti: {}",
                    exit_code_label(&output.status)
                )
            });

        return Err(format_native_build_error(
            path,
            &build_command.display,
            &message,
            &stdout,
            &stderr,
        ));
    }

    let Some(exe) = extract_json_string(&stdout, "exe") else {
        let message = "Build basarili gorundu ama JSON ciktisinda `exe` yolu bulunamadi.";
        return Err(format_native_build_error(
            path,
            &build_command.display,
            message,
            &stdout,
            &stderr,
        ));
    };
    let exe = absolutize_build_output_exe(&exe, build_command.current_dir.as_deref());

    Ok(NativeBuildOutput {
        exe,
        stdout,
        stderr,
    })
}

fn native_build_command(path: &Path) -> NativeBuildCommand {
    if let Some(anadil_exe) = packaged_anadil_cli_path() {
        let current_dir = anadil_exe.parent().map(Path::to_path_buf);
        return NativeBuildCommand {
            display: format!("{} derle --json {}", anadil_exe.display(), path.display()),
            program: anadil_exe,
            args: vec![
                "derle".to_string(),
                "--json".to_string(),
                path.display().to_string(),
            ],
            current_dir,
        };
    }

    NativeBuildCommand {
        program: PathBuf::from("cargo"),
        args: vec![
            "run".to_string(),
            "--bin".to_string(),
            "anadil".to_string(),
            "--".to_string(),
            "derle".to_string(),
            "--json".to_string(),
            path.display().to_string(),
        ],
        current_dir: None,
        display: format!("cargo run --bin anadil -- derle --json {}", path.display()),
    }
}

fn packaged_anadil_cli_path() -> Option<PathBuf> {
    let ide_exe = env::current_exe().ok()?;
    packaged_anadil_cli_path_from_ide_exe(&ide_exe)
}

fn packaged_anadil_cli_path_from_ide_exe(ide_exe: &Path) -> Option<PathBuf> {
    let dir = ide_exe.parent()?;
    let anadil_exe = dir.join("anadil.exe");
    let runtime_lib = dir.join("runtime").join("anadil_runtime.lib");
    (anadil_exe.is_file() && runtime_lib.is_file()).then_some(anadil_exe)
}

fn absolutize_build_output_exe(exe: &str, current_dir: Option<&Path>) -> String {
    let exe_path = PathBuf::from(exe);
    if exe_path.is_absolute() {
        return exe_path.display().to_string();
    }

    current_dir
        .map(|dir| dir.join(exe_path).display().to_string())
        .unwrap_or_else(|| exe.to_string())
}

fn run_executable(path: &Path) -> Result<std::process::Output, std::io::Error> {
    let mut command = Command::new(path);
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    hide_command_window(&mut command);
    command.output()
}

fn format_program_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();

    if stdout.is_empty() && stderr.is_empty() {
        "Program cikti uretmedi.".to_string()
    } else if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        format!("stderr:\n{stderr}")
    } else {
        format!("{stdout}\n\nstderr:\n{stderr}")
    }
}

fn format_exe_run_output(path: &Path, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();

    format!(
        "{}\nDosya: {}\nExit: {}\n\n{}\n{}\n\n{}\n{}",
        section_header_text("Program"),
        path.display(),
        exit_code_label(&output.status),
        section_header_text("stdout"),
        empty_label(stdout),
        section_header_text("stderr"),
        empty_label(stderr),
    )
}

fn format_no_exe_to_run() -> String {
    format!(
        "{}\n{}\n\n{}",
        section_header_text("EXE Calistir"),
        "Calistirilacak executable yok.",
        "Once `EXE Derle` ile native executable uret."
    )
}

fn section_header_text(title: &str) -> String {
    format!("{title}\n{}", "-".repeat(title.chars().count()))
}

fn format_build_started(path: &Path) -> String {
    format!(
        "{}\nKaynak: {}\nDurum: Derleme baslatildi...",
        section_header_text("EXE Derle"),
        path.display(),
    )
}

fn format_build_success(path: &Path, build: &NativeBuildOutput) -> String {
    format!(
        "{}\nDurum: Basarili\nKaynak: {}\nExecutable: {}\n\n{}\n{}\n\n{}\n{}",
        section_header_text("EXE Derle"),
        path.display(),
        build.exe,
        section_header_text("Derleyici stdout"),
        empty_label(build.stdout.trim_end()),
        section_header_text("Derleyici stderr"),
        empty_label(build.stderr.trim_end()),
    )
}

fn format_native_build_error(
    path: &Path,
    command: &str,
    message: &str,
    stdout: &str,
    stderr: &str,
) -> String {
    format!(
        "{}\nDurum: Basarisiz\nKaynak: {}\nKomut: {}\n\n{}\n{}\n\n{}\n{}\n\n{}\n{}\n\n{}\n{}",
        section_header_text("EXE Derle"),
        path.display(),
        command,
        section_header_text("Hata"),
        empty_label(message.trim()),
        section_header_text("Ne yapmali"),
        native_build_advice(message),
        section_header_text("Derleyici stdout"),
        empty_label(stdout.trim_end()),
        section_header_text("Derleyici stderr"),
        empty_label(stderr.trim_end()),
    )
}

fn native_build_advice(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("ml64") || lower.contains("masm") || lower.contains("link.exe") {
        "Visual Studio Build Tools C++ araclari kurulu ve erisilebilir olmali. Gerekirse Build Tools kurulumunda C++ build tools secenegini kontrol et."
    } else if lower.contains("cannot open file")
        || lower.contains("dosya")
        || lower.contains("path")
        || lower.contains("masa")
    {
        "Kaynak yolunda bosluk/Turkce karakter/OneDrive etkisi olabilir. Dosyayi proje klasoru icinde kaydedip tekrar dene; hata surerse Build sekmesindeki ham stdout/stderr'i kullan."
    } else if lower.contains("entry") || lower.contains("ana") {
        "Programda `Ana()` giris noktasi ve semantic hatalarini kontrol et."
    } else {
        "Diagnostics sekmesini ve Build sekmesindeki stdout/stderr detaylarini kontrol et. Mesaj toolchain kaynakliysa Visual Studio Build Tools kurulumu ilk supheli."
    }
}

fn non_empty_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let line_number = line_index + 1;
        let background = if error_lines.contains(&line_number) {
            ERR_LINE_BG
        } else {
            Color32::TRANSPARENT
        };

        let body = line.trim_end_matches(['\r', '\n']);
        let ending = &line[body.len()..];
        append_highlighted_line(&mut job, body, background);
        if !ending.is_empty() {
            job.append(ending, 0.0, format(FG_PRIMARY, background));
        }
    }

    if source.is_empty() {
        job.append("", 0.0, format(FG_PRIMARY, Color32::TRANSPARENT));
    }

    job
}

fn append_highlighted_line(job: &mut LayoutJob, line: &str, background: Color32) {
    let mut index = 0;
    let chars = line.char_indices().collect::<Vec<_>>();

    while index < line.len() {
        let rest = &line[index..];
        if rest.starts_with("//") {
            job.append(rest, 0.0, format(SYN_COMMENT, background));
            break;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };

        if ch == '"' {
            let end = string_end(rest);
            let token = &rest[..end];
            job.append(token, 0.0, format(SYN_STRING, background));
            index += token.len();
        } else if ch.is_ascii_digit() {
            let end = take_while(rest, |c| c.is_ascii_digit());
            let token = &rest[..end];
            job.append(token, 0.0, format(SYN_NUMBER, background));
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
            job.append(token, 0.0, format(SYN_PUNCT, background));
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
        return format(SYN_KEYWORD, background);
    }

    if matches!(token, "sayı" | "mantık" | "metin") {
        return format(SYN_TYPE, background);
    }

    if matches!(token, "Ana" | "yazdır" | "yazdir" | "uzunluk") {
        return format(SYN_BUILTIN, background);
    }

    if token.chars().next().is_some_and(char::is_uppercase) {
        return format(SYN_FUNCTION, background);
    }

    format(FG_PRIMARY, background)
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
Ana() {

}
"
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        apply_editor_smart_edit, char_index_for_line_column, editor_line_count,
        format_build_started, format_ide_state, format_native_build_error, highlight_job,
        list_project_files, native_build_advice, native_build_command,
        packaged_anadil_cli_path_from_ide_exe, parent_hint, parse_ide_state, project_child_path,
        relative_component_depth, sibling_file_path, AnadilIde, IdeSavedState, BOTTOM_PANEL_MIN,
        LEFT_PANEL_MAX,
    };
    use std::{fs, path::Path};

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
    fn syntax_layout_preserves_editor_text() {
        for source in ["", "Ana() {\n\n}", "Ana() {\r\n    yazdır(1);\r\n}\r\n"] {
            assert_eq!(highlight_job(source, &[]).text, source);
        }
    }

    #[test]
    fn editor_smart_edit_closes_pairs() {
        let previous = "Ana";
        let mut current = "Ana(".to_string();
        let cursor = apply_editor_smart_edit(previous, &mut current, 4);

        assert_eq!(current, "Ana()");
        assert_eq!(cursor, Some(4));
    }

    #[test]
    fn editor_smart_edit_indents_after_newline() {
        let previous = "Ana() {";
        let mut current = "Ana() {\n".to_string();
        let cursor = apply_editor_smart_edit(previous, &mut current, 8);

        assert_eq!(current, "Ana() {\n    ");
        assert_eq!(cursor, Some(12));
    }

    #[test]
    fn editor_smart_edit_expands_brace_block() {
        let previous = "Ana() {}";
        let mut current = "Ana() {\n}".to_string();
        let cursor = apply_editor_smart_edit(previous, &mut current, 8);

        assert_eq!(current, "Ana() {\n    \n}");
        assert_eq!(cursor, Some(12));
    }

    #[test]
    fn parses_and_formats_ide_state() {
        let root = Path::new(r"C:\projeler\anadil");
        let file = root.join("main.ana");
        let source = format_ide_state(Some(root), Some(&file), Some(310.0), Some(240.0));
        let parsed = parse_ide_state(&source);

        assert_eq!(parsed.project_root.as_deref(), Some(root));
        assert_eq!(parsed.current_path.as_deref(), Some(file.as_path()));
        assert_eq!(parsed.left_panel_width, Some(310.0));
        assert_eq!(parsed.bottom_panel_height, Some(240.0));
    }

    #[test]
    fn describes_relative_file_depth() {
        assert_eq!(relative_component_depth("main.ana"), 1);
        assert_eq!(relative_component_depth(r"src\main.ana"), 2);
        assert_eq!(parent_hint(r"src\main.ana"), "(src)");
        assert_eq!(parent_hint("main.ana"), "");
    }

    #[test]
    fn project_file_list_recurses_and_skips_ignored_dirs() {
        let root = Path::new("target").join("native_ide_project_files_test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src").join("nested")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("src").join("main.ana"), "").unwrap();
        fs::write(root.join("src").join("nested").join("lib.ana"), "").unwrap();
        fs::write(root.join(".git").join("hidden.ana"), "").unwrap();
        fs::write(root.join("target").join("generated.ana"), "").unwrap();

        let files = list_project_files(&root)
            .into_iter()
            .map(|path| path.strip_prefix(&root).unwrap().display().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            files,
            vec![
                r"src\main.ana".to_string(),
                r"src\nested\lib.ana".to_string()
            ]
        );
    }

    #[test]
    fn saved_state_ignores_missing_paths_and_clamps_sizes() {
        let mut ide = AnadilIde::default();
        ide.apply_saved_state(IdeSavedState {
            project_root: Some(Path::new("target").join("does-not-exist")),
            current_path: Some(Path::new("target").join("missing.ana")),
            left_panel_width: Some(9999.0),
            bottom_panel_height: Some(1.0),
        });

        assert!(ide.project_root.is_none());
        assert_eq!(ide.current_path, "yeni.ana");
        assert_eq!(ide.left_panel_width, LEFT_PANEL_MAX);
        assert_eq!(ide.bottom_panel_height, BOTTOM_PANEL_MIN);
    }

    #[test]
    fn formats_build_messages_with_source_and_advice() {
        let path = Path::new("examples").join("topla.ana");
        let started = format_build_started(&path);
        assert!(started.contains("EXE Derle"));
        assert!(started.contains("examples"));

        let error = format_native_build_error(
            &path,
            "cargo run --bin anadil -- derle --json examples\\topla.ana",
            "MASM : fatal error A1000: cannot open file",
            "",
            "stderr",
        );
        assert!(error.contains("Durum: Basarisiz"));
        assert!(error.contains("Hata\n----"));
        assert!(error.contains("Ne yapmali"));
        assert!(error.contains("Derleyici stdout"));
    }

    #[test]
    fn packaged_cli_is_resolved_next_to_ide_executable() {
        let dir = Path::new("target").join("ide_command_unit_tests");
        let runtime_dir = dir.join("runtime");
        fs::create_dir_all(&runtime_dir).expect("runtime dir should be created");
        let ide_exe = dir.join("anadil-ide.exe");
        let cli_exe = dir.join("anadil.exe");
        let runtime_lib = runtime_dir.join("anadil_runtime.lib");
        fs::write(&ide_exe, "fake ide").expect("fake ide should be written");
        fs::write(&cli_exe, "fake cli").expect("fake cli should be written");
        fs::write(&runtime_lib, "fake lib").expect("fake runtime lib should be written");

        assert_eq!(
            packaged_anadil_cli_path_from_ide_exe(&ide_exe),
            Some(cli_exe)
        );
    }

    #[test]
    fn dev_native_build_command_uses_cargo_fallback() {
        let command = native_build_command(Path::new("examples").join("topla.ana").as_path());

        assert_eq!(command.program, Path::new("cargo"));
        assert!(command.display.contains("cargo run --bin anadil"));
        assert_eq!(command.current_dir, None);
    }

    #[test]
    fn gives_specific_native_build_advice() {
        assert!(native_build_advice("ml64 not found").contains("Visual Studio Build Tools"));
        assert!(native_build_advice("cannot open file").contains("Kaynak yolunda"));
        assert!(native_build_advice("missing Ana entry").contains("Ana()"));
    }
}
