use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfig {
    pub profile: FoundryConfigProfile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfigProfile {
    pub default: FoundryConfigProfileDefault,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfigProfileDefault {
    pub src: Option<String>,
    pub libs: Option<Vec<String>>,
    pub test: Option<String>,
    pub cache_path: Option<String>,
    pub out: Option<String>,
}
