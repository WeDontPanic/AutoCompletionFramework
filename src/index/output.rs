use serde::{Deserialize, Serialize};

/// A single suggestion in response
#[derive(Serialize, Deserialize)]
pub struct Output {
    pub primary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
}

impl std::fmt::Debug for Output {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OutputItem: {}", self.primary)?;
        if let Some(sec) = &self.secondary {
            write!(f, " ({})", sec)?;
        }
        Ok(())
    }
}

impl Output {
    /// Create a new SuggestionItem
    #[inline]
    pub fn new(primary: String, secondary: Option<String>) -> Self {
        Self { primary, secondary }
    }
}
