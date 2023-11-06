//! Module containing structs to work with languages and translations.
//! To work correctly GraphQL wants to InputObject and OutputObjects to be separate,
//! so TranslationInput and Translation were created.
use std::fmt;

#[derive(GraphQLEnum, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, EnumIterator)]
#[graphql(name = "Language", description = "Applicable Languages")]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[graphql(description = "English")]
    En,
    #[graphql(description = "Chinese")]
    Ch,
    #[graphql(description = "German")]
    De,
    #[graphql(description = "Russian")]
    Ru,
    #[graphql(description = "Spanish")]
    Es,
    #[graphql(description = "French")]
    Fr,
    #[graphql(description = "Korean")]
    Ko,
    #[graphql(description = "Portuguese")]
    Po,
    #[graphql(description = "Japanese")]
    Ja,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lang = match *self {
            Language::En => "en",
            Language::Ch => "ch",
            Language::De => "de",
            Language::Ru => "ru",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::Ko => "ko",
            Language::Po => "po",
            Language::Ja => "ja",
        };
        write!(f, "{}", lang)
    }
}

impl Language {
    pub fn as_vec() -> Vec<LanguageGraphQl> {
        Language::enum_iter().map(|value| LanguageGraphQl::new(value.to_string())).collect()
    }
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[graphql(description = "Text with language")]
pub struct TranslationInput {
    #[graphql(description = "Language")]
    pub lang: Language,
    #[graphql(description = "Text")]
    pub text: String,
}

#[derive(GraphQLObject, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[graphql(description = "Text with language")]
pub struct Translation {
    #[graphql(description = "Language")]
    pub lang: Language,
    #[graphql(description = "Text")]
    pub text: String,
}

impl Translation {
    pub fn new(lang: Language, text: String) -> Self {
        Self { lang, text }
    }
}

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
pub struct LanguageGraphQl {
    #[graphql(description = "ISO 639-1 code")]
    pub iso_code: String,
}

impl LanguageGraphQl {
    pub fn new(iso_code: String) -> Self {
        Self { iso_code }
    }
}
