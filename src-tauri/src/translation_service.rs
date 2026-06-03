//! Translation Service for Exodus Browser
//! 
//! This module provides translation capabilities using external translation APIs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Supported languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Chinese,
    Spanish,
    French,
    German,
    Japanese,
    Korean,
    Russian,
    Portuguese,
    Italian,
    Dutch,
    Arabic,
    Hindi,
    Turkish,
    Vietnamese,
    Thai,
    Indonesian,
    Malay,
    Filipino,
    Swedish,
    Norwegian,
    Danish,
    Finnish,
    Polish,
    Czech,
    Greek,
    Hebrew,
    Tibetan,
    Ukrainian,
    Romanian,
    Hungarian,
    Bulgarian,
    Slovak,
    Croatian,
    Serbian,
    Slovenian,
    Lithuanian,
    Latvian,
    Estonian,
    Icelandic,
    Irish,
    Welsh,
    ScottishGaelic,
    Belarusian,
    Kazakh,
    Uzbek,
    Azerbaijani,
    Georgian,
    Armenian,
    Mongolian,
    Nepali,
    Bengali,
    Tamil,
    Telugu,
    Marathi,
    Kannada,
    Malayalam,
    Sinhala,
    Burmese,
    Khmer,
    Lao,
    Amharic,
    Swahili,
    Zulu,
    Xhosa,
    Afrikaans,
    Yoruba,
    Hausa,
    Igbo,
    Somali,
    Oromo,
    Tigrinya,
    Punjabi,
    Gujarati,
    Urdu,
    Persian,
    Pashto,
    Tajik,
    Kyrgyz,
    Turkmen,
    Uyghur,
    Dzongkha,
    Bhutanese,
    Divehi,
    Odia,
    Assamese,
    Maithili,
    Santali,
    Kashmiri,
    Dogri,
    Manipuri,
    Bodo,
    Khasi,
    Garo,
    Mizo,
    Naga,
    Kuki,
    Meitei,
    Tripuri,
    Sikkimese,
    Lepcha,
    Bhutia,
    Tamang,
    Gurung,
    Magar,
    Tharu,
    Danuwar,
    Rai,
    Limbu,
    Sunuwar,
    Jirel,
    Walung,
    Thakali,
    Auto,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "en" | "english" => Language::English,
            "zh" | "chinese" => Language::Chinese,
            "es" | "spanish" => Language::Spanish,
            "fr" | "french" => Language::French,
            "de" | "german" => Language::German,
            "ja" | "japanese" => Language::Japanese,
            "ko" | "korean" => Language::Korean,
            "ru" | "russian" => Language::Russian,
            "pt" | "portuguese" => Language::Portuguese,
            "it" | "italian" => Language::Italian,
            "nl" | "dutch" => Language::Dutch,
            "ar" | "arabic" => Language::Arabic,
            "hi" | "hindi" => Language::Hindi,
            "tr" | "turkish" => Language::Turkish,
            "vi" | "vietnamese" => Language::Vietnamese,
            "th" | "thai" => Language::Thai,
            "id" | "indonesian" => Language::Indonesian,
            "ms" | "malay" => Language::Malay,
            "tl" | "filipino" => Language::Filipino,
            "sv" | "swedish" => Language::Swedish,
            "no" | "norwegian" => Language::Norwegian,
            "da" | "danish" => Language::Danish,
            "fi" | "finnish" => Language::Finnish,
            "pl" | "polish" => Language::Polish,
            "cs" | "czech" => Language::Czech,
            "el" | "greek" => Language::Greek,
            "he" | "hebrew" => Language::Hebrew,
            "uk" | "ukrainian" => Language::Ukrainian,
            "ro" | "romanian" => Language::Romanian,
            "hu" | "hungarian" => Language::Hungarian,
            "bg" | "bulgarian" => Language::Bulgarian,
            "sk" | "slovak" => Language::Slovak,
            "hr" | "croatian" => Language::Croatian,
            "sr" | "serbian" => Language::Serbian,
            "sl" | "slovenian" => Language::Slovenian,
            "lt" | "lithuanian" => Language::Lithuanian,
            "lv" | "latvian" => Language::Latvian,
            "et" | "estonian" => Language::Estonian,
            "is" | "icelandic" => Language::Icelandic,
            "ga" | "irish" => Language::Irish,
            "cy" | "welsh" => Language::Welsh,
            "gd" | "scottish_gaelic" => Language::ScottishGaelic,
            "be" | "belarusian" => Language::Belarusian,
            "kk" | "kazakh" => Language::Kazakh,
            "uz" | "uzbek" => Language::Uzbek,
            "az" | "azerbaijani" => Language::Azerbaijani,
            "ka" | "georgian" => Language::Georgian,
            "hy" | "armenian" => Language::Armenian,
            "mn" | "mongolian" => Language::Mongolian,
            "ne" | "nepali" => Language::Nepali,
            "bn" | "bengali" => Language::Bengali,
            "ta" | "tamil" => Language::Tamil,
            "te" | "telugu" => Language::Telugu,
            "mr" | "marathi" => Language::Marathi,
            "kn" | "kannada" => Language::Kannada,
            "ml" | "malayalam" => Language::Malayalam,
            "si" | "sinhala" => Language::Sinhala,
            "my" | "burmese" => Language::Burmese,
            "km" | "khmer" => Language::Khmer,
            "lo" | "lao" => Language::Lao,
            "am" | "amharic" => Language::Amharic,
            "sw" | "swahili" => Language::Swahili,
            "zu" | "zulu" => Language::Zulu,
            "xh" | "xhosa" => Language::Xhosa,
            "af" | "afrikaans" => Language::Afrikaans,
            "yo" | "yoruba" => Language::Yoruba,
            "ha" | "hausa" => Language::Hausa,
            "ig" | "igbo" => Language::Igbo,
            "so" | "somali" => Language::Somali,
            "om" | "oromo" => Language::Oromo,
            "ti" | "tigrinya" => Language::Tigrinya,
            "pa" | "punjabi" => Language::Punjabi,
            "gu" | "gujarati" => Language::Gujarati,
            "ur" | "urdu" => Language::Urdu,
            "fa" | "persian" => Language::Persian,
            "ps" | "pashto" => Language::Pashto,
            "tg" | "tajik" => Language::Tajik,
            "ky" | "kyrgyz" => Language::Kyrgyz,
            "tk" | "turkmen" => Language::Turkmen,
            "ug" | "uyghur" => Language::Uyghur,
            "bo" | "tibetan" => Language::Tibetan,
            "dz" | "dzongkha" => Language::Dzongkha,
            "dv" | "divehi" => Language::Divehi,
            "or" | "odia" => Language::Odia,
            "as" | "assamese" => Language::Assamese,
            "mai" | "maithili" => Language::Maithili,
            "sat" | "santali" => Language::Santali,
            "ks" | "kashmiri" => Language::Kashmiri,
            "doi" | "dogri" => Language::Dogri,
            "mni" | "manipuri" => Language::Manipuri,
            "brx" | "bodo" => Language::Bodo,
            "kha" | "khasi" => Language::Khasi,
            "gar" | "garo" => Language::Garo,
            "mzo" | "mizo" => Language::Mizo,
            "auto" => Language::Auto,
            _ => Language::English,
        }
    }
    
    pub fn to_code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Dutch => "nl",
            Language::Arabic => "ar",
            Language::Hindi => "hi",
            Language::Turkish => "tr",
            Language::Vietnamese => "vi",
            Language::Thai => "th",
            Language::Indonesian => "id",
            Language::Malay => "ms",
            Language::Filipino => "tl",
            Language::Swedish => "sv",
            Language::Norwegian => "no",
            Language::Danish => "da",
            Language::Finnish => "fi",
            Language::Polish => "pl",
            Language::Czech => "cs",
            Language::Greek => "el",
            Language::Hebrew => "he",
            Language::Ukrainian => "uk",
            Language::Romanian => "ro",
            Language::Hungarian => "hu",
            Language::Bulgarian => "bg",
            Language::Slovak => "sk",
            Language::Croatian => "hr",
            Language::Serbian => "sr",
            Language::Slovenian => "sl",
            Language::Lithuanian => "lt",
            Language::Latvian => "lv",
            Language::Estonian => "et",
            Language::Icelandic => "is",
            Language::Irish => "ga",
            Language::Welsh => "cy",
            Language::ScottishGaelic => "gd",
            Language::Belarusian => "be",
            Language::Kazakh => "kk",
            Language::Uzbek => "uz",
            Language::Azerbaijani => "az",
            Language::Georgian => "ka",
            Language::Armenian => "hy",
            Language::Mongolian => "mn",
            Language::Nepali => "ne",
            Language::Bengali => "bn",
            Language::Tamil => "ta",
            Language::Telugu => "te",
            Language::Marathi => "mr",
            Language::Kannada => "kn",
            Language::Malayalam => "ml",
            Language::Sinhala => "si",
            Language::Burmese => "my",
            Language::Khmer => "km",
            Language::Lao => "lo",
            Language::Amharic => "am",
            Language::Swahili => "sw",
            Language::Zulu => "zu",
            Language::Xhosa => "xh",
            Language::Afrikaans => "af",
            Language::Yoruba => "yo",
            Language::Hausa => "ha",
            Language::Igbo => "ig",
            Language::Somali => "so",
            Language::Oromo => "om",
            Language::Tigrinya => "ti",
            Language::Punjabi => "pa",
            Language::Gujarati => "gu",
            Language::Urdu => "ur",
            Language::Persian => "fa",
            Language::Pashto => "ps",
            Language::Tajik => "tg",
            Language::Kyrgyz => "ky",
            Language::Turkmen => "tk",
            Language::Uyghur => "ug",
            Language::Tibetan => "bo",
            Language::Dzongkha => "dz",
            Language::Bhutanese => "dz",
            Language::Divehi => "dv",
            Language::Odia => "or",
            Language::Assamese => "as",
            Language::Maithili => "mai",
            Language::Santali => "sat",
            Language::Kashmiri => "ks",
            Language::Dogri => "doi",
            Language::Manipuri => "mni",
            Language::Bodo => "brx",
            Language::Khasi => "kha",
            Language::Garo => "gar",
            Language::Mizo => "mzo",
            Language::Naga => "naga",
            Language::Kuki => "kuki",
            Language::Meitei => "mni",
            Language::Tripuri => "trip",
            Language::Sikkimese => "skm",
            Language::Lepcha => "lep",
            Language::Bhutia => "bhu",
            Language::Tamang => "tam",
            Language::Gurung => "gru",
            Language::Magar => "mag",
            Language::Tharu => "th",
            Language::Danuwar => "dnw",
            Language::Rai => "rai",
            Language::Limbu => "lim",
            Language::Sunuwar => "suw",
            Language::Jirel => "jir",
            Language::Walung => "wal",
            Language::Thakali => "thk",
            Language::Auto => "auto",
        }
    }
    
    pub fn to_name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "Chinese",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Russian => "Russian",
            Language::Portuguese => "Portuguese",
            Language::Italian => "Italian",
            Language::Dutch => "Dutch",
            Language::Arabic => "Arabic",
            Language::Hindi => "Hindi",
            Language::Turkish => "Turkish",
            Language::Vietnamese => "Vietnamese",
            Language::Thai => "Thai",
            Language::Indonesian => "Indonesian",
            Language::Malay => "Malay",
            Language::Filipino => "Filipino",
            Language::Swedish => "Swedish",
            Language::Norwegian => "Norwegian",
            Language::Danish => "Danish",
            Language::Finnish => "Finnish",
            Language::Polish => "Polish",
            Language::Czech => "Czech",
            Language::Greek => "Greek",
            Language::Hebrew => "Hebrew",
            Language::Tibetan => "Tibetan",
            Language::Ukrainian => "Ukrainian",
            Language::Romanian => "Romanian",
            Language::Hungarian => "Hungarian",
            Language::Bulgarian => "Bulgarian",
            Language::Slovak => "Slovak",
            Language::Croatian => "Croatian",
            Language::Serbian => "Serbian",
            Language::Slovenian => "Slovenian",
            Language::Lithuanian => "Lithuanian",
            Language::Latvian => "Latvian",
            Language::Estonian => "Estonian",
            Language::Icelandic => "Icelandic",
            Language::Irish => "Irish",
            Language::ScottishGaelic => "Scottish Gaelic",
            Language::Welsh => "Welsh",
            Language::Belarusian => "Belarusian",
            Language::Kazakh => "Kazakh",
            Language::Uzbek => "Uzbek",
            Language::Azerbaijani => "Azerbaijani",
            Language::Georgian => "Georgian",
            Language::Armenian => "Armenian",
            Language::Mongolian => "Mongolian",
            Language::Bengali => "Bengali",
            Language::Tamil => "Tamil",
            Language::Telugu => "Telugu",
            Language::Marathi => "Marathi",
            Language::Gujarati => "Gujarati",
            Language::Punjabi => "Punjabi",
            Language::Kannada => "Kannada",
            Language::Malayalam => "Malayalam",
            Language::Odia => "Odia",
            Language::Assamese => "Assamese",
            Language::Maithili => "Maithili",
            Language::Sinhala => "Sinhala",
            Language::Burmese => "Burmese",
            Language::Khmer => "Khmer",
            Language::Lao => "Lao",
            Language::Nepali => "Nepali",
            Language::Dzongkha => "Dzongkha",
            Language::Bhutanese => "Bhutanese",
            Language::Naga => "Naga",
            Language::Kuki => "Kuki",
            Language::Meitei => "Meitei",
            Language::Tripuri => "Tripuri",
            Language::Sikkimese => "Sikkimese",
            Language::Lepcha => "Lepcha",
            Language::Bhutia => "Bhutia",
            Language::Tamang => "Tamang",
            Language::Gurung => "Gurung",
            Language::Magar => "Magar",
            Language::Tharu => "Tharu",
            Language::Danuwar => "Danuwar",
            Language::Rai => "Rai",
            Language::Limbu => "Limbu",
            Language::Sunuwar => "Sunuwar",
            Language::Jirel => "Jirel",
            Language::Walung => "Walung",
            Language::Thakali => "Thakali",
            Language::Divehi => "Divehi",
            Language::Mizo => "Mizo",
            Language::Bodo => "Bodo",
            Language::Santali => "Santali",
            Language::Khasi => "Khasi",
            Language::Garo => "Garo",
            Language::Manipuri => "Manipuri",
            Language::Kashmiri => "Kashmiri",
            Language::Dogri => "Dogri",
            Language::Pashto => "Pashto",
            Language::Urdu => "Urdu",
            Language::Persian => "Persian",
            Language::Tajik => "Tajik",
            Language::Kyrgyz => "Kyrgyz",
            Language::Turkmen => "Turkmen",
            Language::Uyghur => "Uyghur",
            Language::Oromo => "Oromo",
            Language::Tigrinya => "Tigrinya",
            Language::Amharic => "Amharic",
            Language::Somali => "Somali",
            Language::Swahili => "Swahili",
            Language::Yoruba => "Yoruba",
            Language::Igbo => "Igbo",
            Language::Hausa => "Hausa",
            Language::Zulu => "Zulu",
            Language::Xhosa => "Xhosa",
            Language::Afrikaans => "Afrikaans",
            Language::Auto => "Auto Detect",
        }
    }
}

/// Translation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// Original text
    pub original_text: String,
    /// Translated text
    pub translated_text: String,
    /// Source language
    pub source_language: Language,
    /// Target language
    pub target_language: Language,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Translation service settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationSettings {
    /// Enable auto-translation
    pub auto_translate: bool,
    /// Default target language
    pub default_target_language: Language,
    /// Enable translation of specific elements
    pub translate_text: bool,
    pub translate_alt_text: bool,
    pub translate_title: bool,
}

impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            auto_translate: false,
            default_target_language: Language::English,
            translate_text: true,
            translate_alt_text: true,
            translate_title: true,
        }
    }
}

/// Translation service
pub struct TranslationService {
    settings: Arc<Mutex<TranslationSettings>>,
    cache: Arc<Mutex<HashMap<String, TranslationResult>>>,
}

impl TranslationService {
    /// Create a new translation service
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(TranslationSettings::default())),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get supported languages
    #[allow(dead_code)]
    pub fn get_supported_languages() -> Vec<(String, String, String)> {
        vec![
            ("en".to_string(), "English".to_string(), "🇬🇧".to_string()),
            ("zh".to_string(), "Chinese".to_string(), "🇨🇳".to_string()),
            ("es".to_string(), "Spanish".to_string(), "🇪🇸".to_string()),
            ("fr".to_string(), "French".to_string(), "🇫🇷".to_string()),
            ("de".to_string(), "German".to_string(), "🇩🇪".to_string()),
            ("ja".to_string(), "Japanese".to_string(), "🇯🇵".to_string()),
            ("ko".to_string(), "Korean".to_string(), "🇰🇷".to_string()),
            ("ru".to_string(), "Russian".to_string(), "🇷🇺".to_string()),
            ("pt".to_string(), "Portuguese".to_string(), "🇵🇹".to_string()),
            ("it".to_string(), "Italian".to_string(), "🇮🇹".to_string()),
            ("nl".to_string(), "Dutch".to_string(), "🇳🇱".to_string()),
            ("ar".to_string(), "Arabic".to_string(), "🇸🇦".to_string()),
            ("hi".to_string(), "Hindi".to_string(), "🇮🇳".to_string()),
            ("tr".to_string(), "Turkish".to_string(), "🇹🇷".to_string()),
            ("vi".to_string(), "Vietnamese".to_string(), "🇻🇳".to_string()),
            ("th".to_string(), "Thai".to_string(), "🇹🇭".to_string()),
            ("id".to_string(), "Indonesian".to_string(), "🇮🇩".to_string()),
            ("ms".to_string(), "Malay".to_string(), "🇲🇾".to_string()),
            ("tl".to_string(), "Filipino".to_string(), "🇵🇭".to_string()),
            ("sv".to_string(), "Swedish".to_string(), "🇸🇪".to_string()),
            ("no".to_string(), "Norwegian".to_string(), "🇳🇴".to_string()),
            ("da".to_string(), "Danish".to_string(), "🇩🇰".to_string()),
            ("fi".to_string(), "Finnish".to_string(), "🇫🇮".to_string()),
            ("pl".to_string(), "Polish".to_string(), "🇵🇱".to_string()),
            ("cs".to_string(), "Czech".to_string(), "🇨🇿".to_string()),
            ("el".to_string(), "Greek".to_string(), "🇬🇷".to_string()),
            ("he".to_string(), "Hebrew".to_string(), "🇮🇱".to_string()),
            ("bo".to_string(), "Tibetan".to_string(), "🇹🇮".to_string()),
        ]
    }
    
    /// Translate text
    pub fn translate(&self, text: String, target_lang: Language) -> Result<TranslationResult, String> {
        // Check cache first
        let cache_key = format!("{}:{}", text, target_lang.to_code());
        if let Some(cached) = self.cache.lock()
            .ok()
            .and_then(|cache| cache.get(&cache_key).cloned())
        {
            return Ok(cached);
        }
        
        // Detect source language (simplified - in real implementation would use ML)
        let source_lang = Language::English; // Default to English for now
        
        // Perform translation (simplified - in real implementation would call external API)
        let translated_text = self.translate_text(&text, &source_lang, &target_lang)?;
        
        let result = TranslationResult {
            original_text: text.clone(),
            translated_text,
            source_language: source_lang,
            target_language: target_lang,
            confidence: 0.8, // Default confidence
        };
        
        // Cache the result
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, result.clone());
        }
        
        Ok(result)
    }
    
    /// Translate text (simplified implementation)
    fn translate_text(&self, text: &str, _source: &Language, target: &Language) -> Result<String, String> {
        // In a real implementation, this would call an external translation API
        // For now, we'll return a placeholder with the target language info
        
        let lang_name = target.to_name();
        
        Ok(format!("[Translated to {}] {}", lang_name, text))
    }
    
    /// Get translation settings
    pub fn get_settings(&self) -> TranslationSettings {
        self.settings.lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }
    
    /// Update translation settings
    pub fn update_settings(&self, settings: TranslationSettings) {
        if let Ok(mut current) = self.settings.lock() {
            *current = settings;
        }
    }
    
    /// Clear translation cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
    
}

// Tauri Commands

/// Translate text
#[tauri::command]
pub fn translate_text(
    text: String,
    target_lang: String,
    service: State<'_, Arc<TranslationService>>,
) -> Result<TranslationResult, String> {
    let target_language = Language::from_code(&target_lang);
    service.translate(text, target_language)
}

/// Get translation settings
#[tauri::command]
pub fn get_translation_settings(
    service: State<'_, Arc<TranslationService>>,
) -> Result<TranslationSettings, String> {
    Ok(service.get_settings())
}

/// Update translation settings
#[tauri::command]
pub fn update_translation_settings(
    settings: TranslationSettings,
    service: State<'_, Arc<TranslationService>>,
) -> Result<(), String> {
    service.update_settings(settings);
    Ok(())
}

/// Clear translation cache
#[tauri::command]
pub fn clear_translation_cache(
    service: State<'_, Arc<TranslationService>>,
) -> Result<(), String> {
    service.clear_cache();
    Ok(())
}

/// Get supported languages
#[allow(dead_code)]
#[tauri::command]
pub fn get_supported_languages() -> Result<Vec<(String, String, String)>, String> {
    Ok(TranslationService::get_supported_languages())
}

