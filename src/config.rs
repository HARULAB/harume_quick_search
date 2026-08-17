use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub width: f32,
    pub height: f32,
    pub use_counts: HashMap<String, u32>,
    #[serde(default)]
    pub language: Language,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 850.0,
            height: 900.0,
            use_counts: HashMap::new(),
            language: Language::Japanese,
        }
    }
}

/// 設定ファイルのパス。
///
/// 本体が返すアプリケーションデータフォルダ配下の `quick_search/` に置く。
/// 以前は `C:\ProgramData\aviutl2\Plugin` をハードコードしていたため、
/// - au2 の隔離環境でテストしても本番の設定・使用頻度を書き換えてしまう
/// - 本体がプラグインを走査する `Plugin/` に設定JSONが混ざる
/// という問題があった。`app_data_path()` は隔離環境では隔離側を返す。
pub fn get_config_path() -> std::path::PathBuf {
    let mut path = aviutl2::config::app_data_path();
    path.push("quick_search");
    if let Err(e) = std::fs::create_dir_all(&path) {
        aviutl2::lprintln!(
            "QuickSearch: 設定フォルダを作成できませんでした ({}): {}",
            path.display(),
            e
        );
    }
    path.push("settings.json");
    path
}

/// 旧バージョンの設定ファイル（Pluginフォルダ直下）。初回のみ移行に使う。
fn legacy_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from(r"C:\ProgramData\aviutl2\Plugin\quick_search_settings.json")
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // 旧パスから引き継ぐ
            match std::fs::read_to_string(legacy_config_path()) {
                Ok(s) => {
                    aviutl2::lprintln!("QuickSearch: 旧パスの設定を移行しました");
                    Some(s)
                }
                Err(_) => None,
            }
        }
        Err(e) => {
            aviutl2::lprintln!(
                "QuickSearch: 設定を読み込めませんでした ({}): {}",
                path.display(),
                e
            );
            None
        }
    };

    match raw {
        Some(s) => match serde_json::from_str(&s) {
            Ok(cfg) => cfg,
            Err(e) => {
                aviutl2::lprintln!(
                    "QuickSearch: 設定の解析に失敗したため既定値を使います: {}",
                    e
                );
                AppConfig::default()
            }
        },
        None => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) {
    let path = get_config_path();
    match serde_json::to_string_pretty(config) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                aviutl2::lprintln!(
                    "QuickSearch: 設定を保存できませんでした ({}): {}",
                    path.display(),
                    e
                );
            }
        }
        Err(e) => aviutl2::lprintln!("QuickSearch: 設定のシリアライズに失敗しました: {}", e),
    }
}
