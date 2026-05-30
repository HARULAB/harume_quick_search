use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::GLOBAL_EDIT_HANDLE;

// ── Static state ──────────────────────────────────────────────────────────────

static THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static SELF_HWND: AtomicIsize = AtomicIsize::new(0);
static WANTS_FOCUS: AtomicBool = AtomicBool::new(false);
static WANTS_REFRESH: AtomicBool = AtomicBool::new(false);
static WANTS_SETTINGS: AtomicBool = AtomicBool::new(false);

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppConfig {
    width: f32,
    height: f32,
    use_counts: HashMap<String, u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 850.0,
            height: 900.0,
            use_counts: HashMap::new(),
        }
    }
}

fn get_config_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(r"C:\ProgramData\aviutl2\Plugin");
    let _ = std::fs::create_dir_all(&p);
    p.push("quick_search_settings.json");
    p
}

fn load_config() -> AppConfig {
    std::fs::read_to_string(get_config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(config: &AppConfig) {
    if let Ok(s) = serde_json::to_string(config) {
        let _ = std::fs::write(get_config_path(), s);
    }
}

// ── Effect types ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectCategory {
    All,
    Filter,
    Object,
    Scene,
}

pub const CATEGORIES: [EffectCategory; 4] = [
    EffectCategory::All,
    EffectCategory::Filter,
    EffectCategory::Object,
    EffectCategory::Scene,
];

impl EffectCategory {
    fn label(self) -> &'static str {
        match self {
            Self::All => "すべて",
            Self::Filter => "フィルタ",
            Self::Object => "オブジェクト",
            Self::Scene => "シーン",
        }
    }

    fn badge_text(self) -> &'static str {
        match self {
            Self::Filter => "FLT",
            Self::Object => "OBJ",
            Self::Scene => "SCN",
            _ => "",
        }
    }

    fn badge_color(self) -> egui::Color32 {
        match self {
            Self::Filter => egui::Color32::from_rgb(130, 165, 220), // 視認性を上げた美しいライトブルー
            Self::Object => egui::Color32::from_rgb(130, 195, 150), // 視認性を上げた美しいソフトグリーン
            Self::Scene => egui::Color32::from_rgb(230, 170, 130),  // 温かみのある美しいソフトアプリコット
            _ => egui::Color32::TRANSPARENT,
        }
    }

    fn badge_bg_color(self) -> egui::Color32 {
        match self {
            // 背景色 (27, 27, 27) に対してほんのわずかに色味を混ぜた、上品でモダンな背景
            Self::Filter => egui::Color32::from_rgb(33, 40, 52),   // わずかに青みがかった暗い色
            Self::Object => egui::Color32::from_rgb(32, 42, 34),   // わずかに緑みがかった暗い色
            Self::Scene => egui::Color32::from_rgb(45, 38, 34),    // わずかに赤・オレンジみがかった暗い色
            _ => egui::Color32::TRANSPARENT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectEntry {
    pub name: String,
    pub name_lower: String,
    pub category: EffectCategory,
}

static ALL_EFFECTS: Mutex<Vec<EffectEntry>> = Mutex::new(Vec::new());

pub fn fetch_effects() {
    if let Ok(mut lock) = ALL_EFFECTS.lock() {
        lock.clear();
        if GLOBAL_EDIT_HANDLE.is_ready() {
            for e in GLOBAL_EDIT_HANDLE.get_effects() {
                let n = &e.name;
                
                let is_scene = n.contains("シーン")
                    || n.contains("scene")
                    || n.contains("Scene");

                let is_object = e.effect_type == aviutl2::generic::EffectType::Input
                    || matches!(e.effect_type, aviutl2::generic::EffectType::Other(4) | aviutl2::generic::EffectType::Other(5))
                    || n.contains("@FIGURE")
                    || n.contains("@LOAD")
                    || n.contains("一時的に保存")
                    || n.contains("保存画像の読込")
                    || n == "テキスト"
                    || n == "図形";

                let cat = if is_scene {
                    EffectCategory::Scene
                } else if is_object {
                    EffectCategory::Object
                } else {
                    EffectCategory::Filter
                };

                lock.push(EffectEntry {
                    name: e.name.clone(),
                    name_lower: e.name.to_lowercase(),
                    category: cat,
                });
            }
        }
    }
}

// ── Win32 show/hide helpers ───────────────────────────────────────────────────

fn win32_hide_window() {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::HWND;

        let hwnd_raw = SELF_HWND.load(Ordering::SeqCst);
        if hwnd_raw == 0 { return; }
        let hwnd = HWND(hwnd_raw as *mut _);
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

fn win32_show_window(parent_hwnd_raw: isize) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::UI::HiDpi::GetDpiForWindow;
        use windows::Win32::Foundation::{HWND, RECT};

        let hwnd_raw = SELF_HWND.load(Ordering::SeqCst);
        if hwnd_raw == 0 { return; }
        let hwnd = HWND(hwnd_raw as *mut _);

        let mut parent = HWND(parent_hwnd_raw as *mut _);
        if parent.0.is_null() { parent = GetDesktopWindow(); }

        let dpi = GetDpiForWindow(parent) as f32;
        let scale = dpi / 96.0;

        let mut rect = RECT::default();
        if GetWindowRect(parent, &mut rect).is_ok() {
            let aw = (rect.right - rect.left) as f32;
            let ah = (rect.bottom - rect.top) as f32;
            
            // 画面サイズに基づいた動的なサイズ計算
            // 横幅：画面の 2/4 (50%)（最小600, 最大1200）
            // 縦幅：画面の 2/3 (66%)（最小500, 最大900）
            let target_w = (aw * 0.50).clamp(600.0 * scale, 1200.0 * scale);
            let target_h = (ah * 0.66).clamp(500.0 * scale, 900.0 * scale);

            let x = (rect.left as f32 + (aw - target_w) / 2.0) as i32;
            let y = (rect.top as f32 + (ah - target_h) / 2.0) as i32;
            let w = target_w as i32;
            let h = target_h as i32;
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }

        let _ = SetForegroundWindow(hwnd);
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn register_and_show() {
    let parent_hwnd_raw = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow().0 as isize
    };

    WANTS_FOCUS.store(true, Ordering::SeqCst);
    WANTS_REFRESH.store(true, Ordering::SeqCst);

    if THREAD_STARTED.load(Ordering::SeqCst) {
        fetch_effects();
        win32_show_window(parent_hwnd_raw);
        return;
    }

    fetch_effects();
    THREAD_STARTED.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        let cfg = load_config();
        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size([cfg.width, cfg.height])
            .with_title("Quick Search")
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true);

        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetDesktopWindow};
            use windows::Win32::UI::HiDpi::GetDpiForWindow;
            use windows::Win32::Foundation::{RECT, HWND};
            let mut rect = RECT::default();
            let mut parent = HWND(parent_hwnd_raw as *mut _);
            if parent.0.is_null() { parent = GetDesktopWindow(); }
            if GetWindowRect(parent, &mut rect).is_ok() {
                let dpi = GetDpiForWindow(parent) as f32;
                let scale = dpi / 96.0;
                let aw = (rect.right - rect.left) as f32;
                let ah = (rect.bottom - rect.top) as f32;
                let x = (rect.left as f32 + (aw - cfg.width * scale) / 2.0) / scale;
                let y = (rect.top as f32 + (ah - cfg.height * scale) / 2.0) / scale;
                viewport = viewport.with_position(egui::pos2(x, y));
            }
        }

        let options = eframe::NativeOptions {
            viewport,
            event_loop_builder: Some(Box::new(|builder| {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                builder.with_any_thread(true);
            })),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "harume_quick_search",
            options,
            Box::new(|cc| {
                setup_japanese_fonts(&cc.egui_ctx);
                setup_custom_style(&cc.egui_ctx);
                Ok(Box::new(HarumeApp::new(cc)))
            }),
        );

        THREAD_STARTED.store(false, Ordering::SeqCst);
        SELF_HWND.store(0, Ordering::SeqCst);
    });
}

pub fn register_and_show_settings() {
    WANTS_SETTINGS.store(true, Ordering::SeqCst);
    register_and_show();
}

// ── Font / Style setup ────────────────────────────────────────────────────────

fn setup_japanese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_paths = [
        "C:\\Windows\\Fonts\\notosansjp-regular.otf",
        "C:\\Windows\\Fonts\\notosansjp-regular.ttf",
        "C:\\Windows\\Fonts\\yuothic.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
    ];
    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert("jp".to_owned(), egui::FontData::from_owned(font_data));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "jp".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("jp".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    style.visuals.window_rounding = egui::Rounding::same(12.0);
    
    let accent = egui::Color32::from_rgb(80, 120, 200);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);
    style.visuals.selection.bg_fill = accent;
    
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional));
    ctx.set_style(style);
}

// ── App struct ────────────────────────────────────────────────────────────────

struct HarumeApp {
    search_query: String,
    selected_category: EffectCategory,
    effects: Vec<EffectEntry>,
    filtered_indices: Vec<usize>,
    keyboard_cursor: Option<usize>,
    scroll_to_cursor: bool,
    config: AppConfig,
    last_save: Instant,
    pending_save: bool,
    dirty_filter: bool,
    focus_search: bool,
    is_composing: bool,
    show_settings: bool,
    settings_status: String,
}

impl HarumeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        
        // ライトモード廃止。常にダークモードを適用
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        let effects = ALL_EFFECTS.lock().map(|l| l.clone()).unwrap_or_default();
        let mut app = Self {
            search_query: String::new(),
            selected_category: EffectCategory::All,
            effects,
            filtered_indices: Vec::new(),
            keyboard_cursor: None,
            scroll_to_cursor: false,
            config,
            last_save: Instant::now(),
            pending_save: false,
            dirty_filter: true,
            focus_search: true,
            is_composing: false,
            show_settings: WANTS_SETTINGS.swap(false, Ordering::SeqCst),
            settings_status: String::new(),
        };
        app.rebuild_filter();

        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::*;
            use windows::core::PCWSTR;
            let title: Vec<u16> = "Quick Search\0".encode_utf16().collect();
            if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
                if !hwnd.0.is_null() {
                    SELF_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
                }
            }
        }
        
        cc.egui_ctx.request_repaint();
        app
    }

    fn rebuild_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        
        let hira_query = romaji_to_hiragana(&query);
        let kata_query = hiragana_to_katakana(&hira_query);

        let words: Vec<String> = query.split_whitespace().map(|w| w.to_string()).collect();
        let hira_words: Vec<String> = hira_query.split_whitespace().map(|w| w.to_string()).collect();
        let kata_words: Vec<String> = kata_query.split_whitespace().map(|w| w.to_string()).collect();

        let mut indices: Vec<usize> = self.effects.iter().enumerate()
            .filter(|(_, e)| {
                let cat_match = self.selected_category == EffectCategory::All || e.category == self.selected_category;
                if !cat_match { return false; }
                
                if words.is_empty() { return true; }
                
                for i in 0..words.len() {
                    let w = &words[i];
                    let hw = &hira_words[i];
                    let kw = &kata_words[i];
                    
                    let matched = e.name_lower.contains(w) 
                        || e.name_lower.contains(hw) 
                        || e.name_lower.contains(kw);
                        
                    if !matched { return false; }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
        
        // 使用頻度 -> カテゴリ -> 名前の順でソート
        indices.sort_by(|&a, &b| {
            let ca = self.config.use_counts.get(&self.effects[a].name).copied().unwrap_or(0);
            let cb = self.config.use_counts.get(&self.effects[b].name).copied().unwrap_or(0);
            
            cb.cmp(&ca)
                .then_with(|| self.effects[a].category.badge_text().cmp(self.effects[b].category.badge_text()))
                .then_with(|| self.effects[a].name.cmp(&self.effects[b].name))
        });
        
        self.filtered_indices = indices;
        self.keyboard_cursor = if self.filtered_indices.is_empty() { None } else { Some(0) };
        self.dirty_filter = false;
        self.scroll_to_cursor = true;
    }

    fn execute_add(&mut self, name: String, category: EffectCategory) {
        request_add_effect(&name, category);
        *self.config.use_counts.entry(name).or_insert(0) += 1;
        save_config(&self.config);
        self.pending_save = false;
    }

    fn hide(&mut self) {
        self.search_query.clear();
        self.keyboard_cursor = None;
        self.show_settings = false;
        win32_hide_window();
    }

    fn reset_use_counts(&mut self) {
        self.config.use_counts.clear();
        save_config(&self.config);
        self.settings_status = "使用回数をリセットしました".to_string();
        self.dirty_filter = true;
    }

    fn reset_window_size(&mut self) {
        self.config.width = AppConfig::default().width;
        self.config.height = AppConfig::default().height;
        save_config(&self.config);
        self.settings_status = "ウィンドウサイズを初期化しました".to_string();
    }

    fn show_settings_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut should_hide = false;

        ui.horizontal(|ui| {
            ui.heading("Quick Search Settings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("閉じる").clicked() {
                    should_hide = true;
                }
                if ui.button("検索に戻る").clicked() {
                    self.show_settings = false;
                    self.focus_search = true;
                }
            });
        });

        ui.add_space(12.0);
        ui.label(format!("設定ファイル: {}", get_config_path().display()));
        ui.add_space(12.0);

        if ui.button("使用回数をリセット").clicked() {
            self.reset_use_counts();
        }

        if ui.button("ウィンドウサイズを初期化").clicked() {
            self.reset_window_size();
        }

        ui.add_space(10.0);
        if !self.settings_status.is_empty() {
            ui.label(&self.settings_status);
        }

        should_hide
    }
}

impl eframe::App for HarumeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if SELF_HWND.load(Ordering::SeqCst) == 0 {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::core::PCWSTR;
                let title: Vec<u16> = "Quick Search\0".encode_utf16().collect();
                if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
                    if !hwnd.0.is_null() {
                        SELF_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
                    }
                }
            }
        }

        if WANTS_REFRESH.swap(false, Ordering::SeqCst) {
            self.effects = ALL_EFFECTS.lock().map(|l| l.clone()).unwrap_or_default();
            self.dirty_filter = true;
        }

        if WANTS_FOCUS.swap(false, Ordering::SeqCst) {
            self.focus_search = true;
        }

        if WANTS_SETTINGS.swap(false, Ordering::SeqCst) {
            self.show_settings = true;
            self.settings_status.clear();
        }

        let mut should_hide = false;
        let mut enter_pressed = false;

        ctx.input(|i| {
            let mut ime_commit_this_frame = false;
            // IME 状態の更新
            for event in &i.events {
                if let egui::Event::Ime(ime_event) = event {
                    match ime_event {
                        egui::ImeEvent::Preedit(text) => self.is_composing = !text.is_empty(),
                        egui::ImeEvent::Commit(_) => {
                            self.is_composing = false;
                            ime_commit_this_frame = true;
                        }
                        egui::ImeEvent::Disabled => self.is_composing = false,
                        _ => {}
                    }
                }
            }

            if i.key_pressed(egui::Key::Escape) { should_hide = true; }
            
            // IME 入力中は確定(Enter)や移動(矢印)を無視する
            if !self.is_composing {
                // Enterが押されたが、それがIMEの確定(Commit)と同じフレームなら、
                // それは「文字の確定」のためのEnterなので無視する。
                if i.key_pressed(egui::Key::Enter) && !ime_commit_this_frame { 
                    enter_pressed = true; 
                }
                
                // 左右矢印でカテゴリ切り替え
                if i.key_pressed(egui::Key::ArrowRight) {
                    let current = CATEGORIES.iter().position(|&c| c == self.selected_category).unwrap_or(0);
                    self.selected_category = CATEGORIES[(current + 1) % CATEGORIES.len()];
                    self.dirty_filter = true;
                }
                if i.key_pressed(egui::Key::ArrowLeft) {
                    let current = CATEGORIES.iter().position(|&c| c == self.selected_category).unwrap_or(0);
                    self.selected_category = CATEGORIES[(current + CATEGORIES.len() - 1) % CATEGORIES.len()];
                    self.dirty_filter = true;
                }

                let old_cursor = self.keyboard_cursor;
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.keyboard_cursor = Some(self.keyboard_cursor.map_or(0, |c| (c + 1).min(self.filtered_indices.len().saturating_sub(1))));
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    self.keyboard_cursor = Some(self.keyboard_cursor.map_or(0, |c| c.saturating_sub(1)));
                }
                if self.keyboard_cursor != old_cursor { self.scroll_to_cursor = true; }
            }
        });

        if enter_pressed {
            if let Some(cursor) = self.keyboard_cursor {
                if let Some(&idx) = self.filtered_indices.get(cursor) {
                    let name = self.effects[idx].name.clone();
                    let cat = self.effects[idx].category;
                    self.execute_add(name, cat);
                    should_hide = true;
                }
            }
        }

        if self.dirty_filter { self.rebuild_filter(); }

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(ctx.style().visuals.window_fill())
                .rounding(ctx.style().visuals.window_rounding)
                .inner_margin(12.0))
            .show(ctx, |ui| {
            if self.show_settings {
                if self.show_settings_ui(ui) {
                    should_hide = true;
                }
                return;
            }
            
            // Header
            egui::Frame::none()
                .fill(ctx.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let res = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("エフェクトを検索...")
                            .frame(false)
                            .desired_width(ui.available_width() - 80.0)
                    );
                    
                    if self.focus_search { 
                        res.request_focus(); 
                        self.focus_search = false; 
                    }
                    
                    if !res.has_focus() && ctx.input(|i| !i.events.is_empty()) {
                        let mut text_typed = false;
                        ctx.input(|i| {
                            for event in &i.events {
                                match event {
                                    egui::Event::Text(_) => text_typed = true,
                                    egui::Event::Ime(egui::ImeEvent::Preedit(_)) => text_typed = true,
                                    _ => {}
                                }
                            }
                        });
                        if text_typed { res.request_focus(); }
                    }

                    if res.changed() { self.dirty_filter = true; }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 絵文字を避けて「閉じる」テキストに変更
                        if ui.add(egui::Button::new(egui::RichText::new("閉じる").size(14.0)).frame(false)).clicked() {
                            should_hide = true;
                        }
                    });
                });
            });

            ui.add_space(8.0);

            // Category Tabs (左寄せ、下線なしの超ミニマル)
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 16.0; // タブ同士の間隔
                for &cat in &CATEGORIES {
                    let selected = self.selected_category == cat;
                    
                    let text_color = if selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(130, 130, 130)
                    };

                    let resp = ui.add(
                        egui::Button::new(
                            egui::RichText::new(cat.label())
                                .color(text_color)
                                .size(14.0)
                                .strong()
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .rounding(0.0)
                        .min_size(egui::vec2(0.0, 26.0))
                    );

                    if resp.clicked() {
                        self.selected_category = cat;
                        self.dirty_filter = true;
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let mut clicked_name = None;
            let mut clicked_cat = EffectCategory::All;

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                for (view_idx, &idx) in self.filtered_indices.iter().enumerate() {
                    let effect = &self.effects[idx];
                    let is_selected = self.keyboard_cursor == Some(view_idx);
                    let is_hovered = is_selected;

                    let response = ui.allocate_response(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
                    let rect = response.rect;
                    let painter = ui.painter();
                    
                    let active = is_hovered || response.hovered();
                    if active {
                        // 背景：白飛びバグが絶対に発生しない安全なダークグレーベタ塗り
                        let bg_color = egui::Color32::from_rgb(36, 36, 36);
                        painter.rect_filled(rect, egui::Rounding::same(4.0), bg_color);
                    }
                    
                    // Badge (トーンダウンしたソフトバッジ)
                    let badge_text = effect.category.badge_text();
                    if !badge_text.is_empty() {
                        let badge_rect = egui::Rect::from_min_size(rect.left_top() + egui::vec2(12.0, 6.0), egui::vec2(36.0, 20.0));
                        painter.rect_filled(badge_rect, egui::Rounding::same(4.0), effect.category.badge_bg_color());
                        painter.text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            badge_text,
                            egui::FontId::new(10.0, egui::FontFamily::Proportional),
                            effect.category.badge_color()
                        );
                    }
                    
                    // テキスト色：ホバー時は白、通常時は目に優しい淡いグレー
                    let text_color = if active {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(180, 180, 180)
                    };
                    
                    painter.text(
                        rect.left_top() + egui::vec2(56.0, 16.0),
                        egui::Align2::LEFT_CENTER,
                        &effect.name,
                        egui::FontId::new(14.0, egui::FontFamily::Proportional),
                        text_color
                    );

                    if is_selected && self.scroll_to_cursor {
                        response.scroll_to_me(None);
                        self.scroll_to_cursor = false;
                    }
                    
                    if response.clicked() {
                        clicked_name = Some(effect.name.clone());
                        clicked_cat = effect.category;
                    }
                }
            });
            
            if let Some(name) = clicked_name {
                self.execute_add(name, clicked_cat);
                should_hide = true;
            }
        });

        if should_hide {
            self.hide();
        }

        // Config saving
        if (ctx.screen_rect().size().x - self.config.width).abs() > 5.0 || (ctx.screen_rect().size().y - self.config.height).abs() > 5.0 {
            self.config.width = ctx.screen_rect().size().x;
            self.config.height = ctx.screen_rect().size().y;
            self.pending_save = true;
        }
        if self.pending_save && self.last_save.elapsed() > Duration::from_secs(1) {
            save_config(&self.config);
            self.pending_save = false;
            self.last_save = Instant::now();
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

// ── AviUtl2 操作 ─────────────────────────────────────────────────────────────

fn request_add_effect(name: &str, category: EffectCategory) {
    let name_clone = name.to_string();
    let _ = GLOBAL_EDIT_HANDLE.call_edit_section(move |edit_section| {
        add_effect_logic(edit_section, &name_clone, category);
    });
}

fn add_effect_logic(edit_section: &mut aviutl2::generic::EditSection, effect_name: &str, category: EffectCategory) {
    // オブジェクト以外は、選択中のオブジェクトがあればフィルタとして追加を試みる
    let is_filter_type = match category {
        EffectCategory::Object => false,
        _ => true,
    };

    if is_filter_type {
        if let Some(target) = find_target_object(edit_section) {
            if add_filter_to_object(edit_section, &target, effect_name) {
                return;
            }
        }
    }
    create_new_object(edit_section, effect_name);
}

fn find_target_object(edit_section: &aviutl2::generic::EditSection) -> Option<aviutl2::generic::ObjectHandle> {
    if let Ok(Some(obj)) = edit_section.get_focused_object() {
        return Some(obj);
    }
    if let Ok(objs) = edit_section.get_selected_objects() {
        if !objs.is_empty() { return Some(objs[0]); }
    }
    None
}

fn add_filter_to_object(edit_section: &mut aviutl2::generic::EditSection, target: &aviutl2::generic::ObjectHandle, effect_name: &str) -> bool {
    if let Ok(alias) = edit_section.get_object_alias(target) {
        let mut max_idx = -1i32;
        for line in alias.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[Object.") {
                if let Some(n) = trimmed.strip_prefix("[Object.").and_then(|s| s.strip_suffix(']')) {
                    if let Ok(i) = n.parse::<i32>() {
                        if i > max_idx { max_idx = i; }
                    }
                }
            }
        }
        
        let new_alias = format!("{}\n[Object.{}]\neffect.name={}\n", alias.trim_end(), max_idx + 1, effect_name);
        
        if let Ok(lf) = edit_section.get_object_layer_frame(target) {
            let _ = edit_section.delete_object(target);
            match edit_section.create_object_from_alias(&new_alias, lf.layer as usize, lf.start as usize, (lf.end - lf.start) as usize) {
                Ok(no) => {
                    let _ = edit_section.focus_object(&no);
                    return true;
                }
                Err(_) => {
                    if let Ok(ro) = edit_section.create_object_from_alias(&alias, lf.layer as usize, lf.start as usize, (lf.end - lf.start) as usize) {
                        let _ = edit_section.focus_object(&ro);
                    }
                    return false;
                }
            }
        }
    }
    false
}

fn create_new_object(edit_section: &mut aviutl2::generic::EditSection, effect_name: &str) {
    let mut layer = edit_section.info.layer as usize;
    let frame = edit_section.info.frame as usize;
    
    for _ in 0..50 {
        match edit_section.create_object(effect_name, layer, frame, Some(180)) {
            Ok(o) => {
                let _ = edit_section.focus_object(&o);
                return;
            }
            Err(_) => {
                layer += 1;
            }
        }
    }
}

pub fn init_plugin_resources() {}

fn romaji_to_hiragana(input: &str) -> String {
    let mut s = input.to_lowercase();
    
    let triples = [
        ("ktsu", "っつ"), ("shya", "しゃ"), ("shyu", "しゅ"), ("shyo", "しょ"),
        ("chya", "ちゃ"), ("chyu", "ちゅ"), ("chyo", "ちょ"), ("jya", "じゃ"),
        ("jyu", "じゅ"), ("jyo", "じょ"), ("tsum", "つむ"),
    ];
    for &(r, h) in &triples { s = s.replace(r, h); }

    let doubles = [
        ("sha", "しゃ"), ("shu", "しゅ"), ("sho", "しょ"),
        ("cha", "ちゃ"), ("chu", "ちゅ"), ("cho", "ちょ"),
        ("chi", "ち"), ("tsu", "つ"), ("shi", "し"),
        ("ka", "か"), ("ki", "き"), ("ku", "く"), ("ke", "け"), ("ko", "こ"),
        ("sa", "さ"), ("si", "し"), ("su", "す"), ("se", "せ"), ("so", "そ"),
        ("ta", "た"), ("ti", "ち"), ("tu", "つ"), ("te", "て"), ("to", "と"),
        ("na", "な"), ("ni", "に"), ("nu", "ぬ"), ("ne", "ね"), ("no", "の"),
        ("ha", "は"), ("hi", "ひ"), ("hu", "ふ"), ("he", "へ"), ("ho", "ほ"),
        ("ma", "ま"), ("mi", "み"), ("mu", "む"), ("me", "め"), ("mo", "も"),
        ("ya", "や"), ("yu", "ゆ"), ("yo", "よ"),
        ("ra", "ら"), ("ri", "り"), ("ru", "る"), ("re", "れ"), ("ro", "ろ"),
        ("wa", "わ"), ("wo", "を"), ("nn", "ん"),
        ("ga", "が"), ("gi", "ぎ"), ("gu", "ぐ"), ("ge", "げ"), ("go", "ご"),
        ("za", "ざ"), ("zi", "じ"), ("zu", "ず"), ("ze", "ぜ"), ("zo", "ぞ"),
        ("da", "だ"), ("di", "ぢ"), ("du", "づ"), ("de", "で"), ("do", "ど"),
        ("ba", "ば"), ("bi", "び"), ("bu", "ぶ"), ("be", "べ"), ("bo", "ぼ"),
        ("pa", "ぱ"), ("pi", "ぴ"), ("pu", "ぷ"), ("pe", "ぺ"), ("po", "ぽ"),
        ("ja", "じゃ"), ("ju", "じゅ"), ("jo", "じょ"), ("fa", "ふぁ"), ("fi", "ふぃ"),
        ("fe", "ふぇ"), ("fo", "ふぉ"), ("wy", "ゐ"), ("ve", "ゔぇ"), ("vu", "ゔ"),
    ];
    for &(r, h) in &doubles { s = s.replace(r, h); }

    let singles = [
        ("a", "あ"), ("i", "い"), ("u", "う"), ("e", "え"), ("o", "お"),
        ("n", "ん"),
    ];
    for &(r, h) in &singles { s = s.replace(r, h); }

    s
}

fn hiragana_to_katakana(input: &str) -> String {
    input.chars().map(|c| {
        let code = c as u32;
        if (0x3041..=0x3096).contains(&code) {
            std::char::from_u32(code + 0x60).unwrap_or(c)
        } else {
            c
        }
    }).collect()
}
