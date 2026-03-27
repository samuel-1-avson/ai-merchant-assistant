use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Arabic,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "es" | "spanish" => Language::Spanish,
            "fr" | "french" => Language::French,
            "de" | "german" => Language::German,
            "zh" | "chinese" => Language::Chinese,
            "ar" | "arabic" => Language::Arabic,
            _ => Language::English,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Chinese => "zh",
            Language::Arabic => "ar",
        }
    }
}

pub struct I18n;

impl I18n {
    pub fn translate(key: &str, lang: Language) -> String {
        let translations = get_translations(lang);
        translations.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn t(key: &str, lang: Language) -> String {
        Self::translate(key, lang)
    }

    /// Format number according to locale
    pub fn format_number(num: f64, lang: Language) -> String {
        match lang {
            Language::Spanish | Language::French | Language::German => {
                // Use comma as decimal separator
                format!("{:.2}", num).replace('.', ",")
            }
            _ => format!("{:.2}", num),
        }
    }

    /// Format currency according to locale
    pub fn format_currency(amount: f64, currency: &str, lang: Language) -> String {
        match lang {
            Language::Spanish => format!("{} {}", amount, currency),
            Language::French => format!("{} {}", amount, currency),
            Language::German => format!("{} {}", amount, currency),
            _ => format!("{}{}", currency, amount),
        }
    }

    /// Format date according to locale
    pub fn format_date(date: &str, lang: Language) -> String {
        // Simple formatting - in production would use chrono-locale
        match lang {
            Language::Spanish => date.to_string(), // DD/MM/YYYY
            Language::French => date.to_string(),  // DD/MM/YYYY
            Language::German => date.to_string(),  // DD.MM.YYYY
            _ => date.to_string(), // MM/DD/YYYY
        }
    }
}

fn get_translations(lang: Language) -> HashMap<&'static str, &'static str> {
    match lang {
        Language::Spanish => spanish_translations(),
        Language::French => french_translations(),
        Language::German => german_translations(),
        Language::Chinese => chinese_translations(),
        Language::Arabic => arabic_translations(),
        _ => english_translations(),
    }
}

fn english_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "Welcome to AI Merchant Assistant");
    map.insert("record_sale", "Record a sale");
    map.insert("total_revenue", "Total Revenue");
    map.insert("transactions", "Transactions");
    map.insert("products_sold", "Products Sold");
    map.insert("low_stock_alert", "Low stock alert");
    map.insert("sales_summary", "Sales Summary");
    map.insert("daily_summary", "Daily Summary");
    map.insert("weekly_summary", "Weekly Summary");
    map.insert("logout", "Logout");
    map.insert("settings", "Settings");
    map.insert("dashboard", "Dashboard");
    map.insert("products", "Products");
    map.insert("analytics", "Analytics");
    map.insert("alerts", "Alerts");
    map
}

fn spanish_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "Bienvenido a AI Merchant Assistant");
    map.insert("record_sale", "Registrar una venta");
    map.insert("total_revenue", "Ingresos Totales");
    map.insert("transactions", "Transacciones");
    map.insert("products_sold", "Productos Vendidos");
    map.insert("low_stock_alert", "Alerta de stock bajo");
    map.insert("sales_summary", "Resumen de Ventas");
    map.insert("daily_summary", "Resumen Diario");
    map.insert("weekly_summary", "Resumen Semanal");
    map.insert("logout", "Cerrar sesión");
    map.insert("settings", "Configuración");
    map.insert("dashboard", "Panel");
    map.insert("products", "Productos");
    map.insert("analytics", "Analíticas");
    map.insert("alerts", "Alertas");
    map
}

fn french_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "Bienvenue sur AI Merchant Assistant");
    map.insert("record_sale", "Enregistrer une vente");
    map.insert("total_revenue", "Revenu Total");
    map.insert("transactions", "Transactions");
    map.insert("products_sold", "Produits Vendus");
    map.insert("low_stock_alert", "Alerte stock faible");
    map.insert("sales_summary", "Résumé des Ventes");
    map.insert("daily_summary", "Résumé Quotidien");
    map.insert("weekly_summary", "Résumé Hebdomadaire");
    map.insert("logout", "Déconnexion");
    map.insert("settings", "Paramètres");
    map.insert("dashboard", "Tableau de Bord");
    map.insert("products", "Produits");
    map.insert("analytics", "Analytiques");
    map.insert("alerts", "Alertes");
    map
}

fn german_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "Willkommen bei AI Merchant Assistant");
    map.insert("record_sale", "Verkauf erfassen");
    map.insert("total_revenue", "Gesamteinnahmen");
    map.insert("transactions", "Transaktionen");
    map.insert("products_sold", "Verkaufte Produkte");
    map.insert("low_stock_alert", "Niedriger Lagerbestand");
    map.insert("sales_summary", "Verkaufsübersicht");
    map.insert("daily_summary", "Tägliche Übersicht");
    map.insert("weekly_summary", "Wöchentliche Übersicht");
    map.insert("logout", "Abmelden");
    map.insert("settings", "Einstellungen");
    map.insert("dashboard", "Übersicht");
    map.insert("products", "Produkte");
    map.insert("analytics", "Analytik");
    map.insert("alerts", "Warnungen");
    map
}

fn chinese_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "欢迎使用 AI Merchant Assistant");
    map.insert("record_sale", "记录销售");
    map.insert("total_revenue", "总收入");
    map.insert("transactions", "交易");
    map.insert("products_sold", "已售产品");
    map.insert("low_stock_alert", "库存不足提醒");
    map.insert("sales_summary", "销售摘要");
    map.insert("daily_summary", "每日摘要");
    map.insert("weekly_summary", "每周摘要");
    map.insert("logout", "退出");
    map.insert("settings", "设置");
    map.insert("dashboard", "仪表板");
    map.insert("products", "产品");
    map.insert("analytics", "分析");
    map.insert("alerts", "提醒");
    map
}

fn arabic_translations() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("welcome", "مرحباً بك في AI Merchant Assistant");
    map.insert("record_sale", "تسجيل عملية بيع");
    map.insert("total_revenue", "إجمالي الإيرادات");
    map.insert("transactions", "المعاملات");
    map.insert("products_sold", "المنتجات المباعة");
    map.insert("low_stock_alert", "تنبيه انخفاض المخزون");
    map.insert("sales_summary", "ملخص المبيعات");
    map.insert("daily_summary", "الملخص اليومي");
    map.insert("weekly_summary", "الملخص الأسبوعي");
    map.insert("logout", "تسجيل الخروج");
    map.insert("settings", "الإعدادات");
    map.insert("dashboard", "لوحة التحكم");
    map.insert("products", "المنتجات");
    map.insert("analytics", "التحليلات");
    map.insert("alerts", "التنبيهات");
    map
}
