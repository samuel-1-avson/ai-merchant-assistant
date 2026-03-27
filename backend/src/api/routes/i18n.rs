use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::AppState;
use crate::i18n::{Language, I18n};

#[derive(Debug, serde::Deserialize)]
pub struct LanguageQuery {
    pub lang: Option<String>,
}

pub async fn get_translations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LanguageQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let lang_code = query.lang.unwrap_or_else(|| "en".to_string());
    let language = Language::from_code(&lang_code);

    let translations = json!({
        "language": language.code(),
        "strings": {
            "welcome": I18n::t("welcome", language),
            "record_sale": I18n::t("record_sale", language),
            "total_revenue": I18n::t("total_revenue", language),
            "transactions": I18n::t("transactions", language),
            "products_sold": I18n::t("products_sold", language),
            "low_stock_alert": I18n::t("low_stock_alert", language),
            "sales_summary": I18n::t("sales_summary", language),
            "daily_summary": I18n::t("daily_summary", language),
            "weekly_summary": I18n::t("weekly_summary", language),
            "logout": I18n::t("logout", language),
            "settings": I18n::t("settings", language),
            "dashboard": I18n::t("dashboard", language),
            "products": I18n::t("products", language),
            "analytics": I18n::t("analytics", language),
            "alerts": I18n::t("alerts", language),
        }
    });

    Ok(Json(json!({
        "success": true,
        "data": translations
    })))
}

pub async fn get_supported_languages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let languages = json!([
        { "code": "en", "name": "English", "native_name": "English" },
        { "code": "es", "name": "Spanish", "native_name": "Español" },
        { "code": "fr", "name": "French", "native_name": "Français" },
        { "code": "de", "name": "German", "native_name": "Deutsch" },
        { "code": "zh", "name": "Chinese", "native_name": "中文" },
        { "code": "ar", "name": "Arabic", "native_name": "العربية" },
    ]);

    Ok(Json(json!({
        "success": true,
        "data": languages
    })))
}

pub async fn format_number(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FormatNumberQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let lang_code = query.lang.unwrap_or_else(|| "en".to_string());
    let language = Language::from_code(&lang_code);

    let formatted = I18n::format_number(query.number, language);

    Ok(Json(json!({
        "success": true,
        "data": {
            "original": query.number,
            "formatted": formatted,
            "language": language.code()
        }
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct FormatNumberQuery {
    pub number: f64,
    pub lang: Option<String>,
}
