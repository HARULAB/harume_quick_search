use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Japanese,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Self::Japanese
    }
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Self::Japanese => "日本語",
            Self::English => "English",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextKey {
    SearchPlaceholder,
    Close,
    BackToSearch,
    SettingsTitle,
    SettingsGeneral,
    SettingsData,
    Language,
    ConfigFile,
    UsageHistory,
    WindowSize,
    ResetUseCounts,
    ResetWindowSize,
    UseCountsReset,
    WindowSizeReset,
    All,
    Filter,
    Object,
    Scene,
}

pub fn tr(language: Language, key: TextKey) -> &'static str {
    match language {
        Language::Japanese => match key {
            TextKey::SearchPlaceholder => "エフェクトを検索...",
            TextKey::Close => "閉じる",
            TextKey::BackToSearch => "検索に戻る",
            TextKey::SettingsTitle => "Quick Search 設定",
            TextKey::SettingsGeneral => "一般",
            TextKey::SettingsData => "データ",
            TextKey::Language => "言語",
            TextKey::ConfigFile => "設定ファイル",
            TextKey::UsageHistory => "使用履歴",
            TextKey::WindowSize => "ウィンドウサイズ",
            TextKey::ResetUseCounts => "使用回数をリセット",
            TextKey::ResetWindowSize => "ウィンドウサイズを初期化",
            TextKey::UseCountsReset => "使用回数をリセットしました",
            TextKey::WindowSizeReset => "ウィンドウサイズを初期化しました",
            TextKey::All => "すべて",
            TextKey::Filter => "フィルタ",
            TextKey::Object => "オブジェクト",
            TextKey::Scene => "シーン",
        },
        Language::English => match key {
            TextKey::SearchPlaceholder => "Search effects...",
            TextKey::Close => "Close",
            TextKey::BackToSearch => "Back to Search",
            TextKey::SettingsTitle => "Quick Search Settings",
            TextKey::SettingsGeneral => "General",
            TextKey::SettingsData => "Data",
            TextKey::Language => "Language",
            TextKey::ConfigFile => "Settings file",
            TextKey::UsageHistory => "Usage History",
            TextKey::WindowSize => "Window Size",
            TextKey::ResetUseCounts => "Reset Usage Counts",
            TextKey::ResetWindowSize => "Reset Window Size",
            TextKey::UseCountsReset => "Usage counts have been reset",
            TextKey::WindowSizeReset => "Window size has been reset",
            TextKey::All => "All",
            TextKey::Filter => "Filter",
            TextKey::Object => "Object",
            TextKey::Scene => "Scene",
        },
    }
}
