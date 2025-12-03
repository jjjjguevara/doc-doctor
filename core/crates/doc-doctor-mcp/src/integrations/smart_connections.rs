//! Smart Connections Integration
//!
//! Provides semantic search and RAG capabilities using Smart Connections embeddings.
//! Smart Connections stores embeddings in `.smart-env/` or `.smart-connections/` folders.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::IntegrationStatus;

/// Smart Connections integration for semantic search
pub struct SmartConnectionsIntegration {
    /// Path to the vault
    vault_path: Option<PathBuf>,
    /// Path to the embeddings data
    env_path: Option<PathBuf>,
    /// Cached embeddings (path -> embedding vector)
    embeddings: HashMap<String, Vec<f32>>,
    /// Whether embeddings are loaded
    loaded: bool,
}

impl SmartConnectionsIntegration {
    /// Create a new Smart Connections integration
    pub fn new() -> Self {
        Self {
            vault_path: None,
            env_path: None,
            embeddings: HashMap::new(),
            loaded: false,
        }
    }

    /// Create for a specific vault path
    pub fn for_vault(vault_path: &Path) -> Self {
        let env_path = Self::find_env_path(vault_path);
        Self {
            vault_path: Some(vault_path.to_path_buf()),
            env_path,
            embeddings: HashMap::new(),
            loaded: false,
        }
    }

    /// Check if Smart Connections is available
    pub fn check_availability(&self) -> IntegrationStatus {
        if self.env_path.is_some() {
            IntegrationStatus::available("Smart Connections")
        } else {
            IntegrationStatus::unavailable(
                "Smart Connections",
                "Smart Connections plugin not detected. Install from Obsidian community plugins for semantic search.",
            )
        }
    }

    /// Find the Smart Connections environment path
    fn find_env_path(vault_path: &Path) -> Option<PathBuf> {
        // Check common locations
        let candidates = [
            vault_path.join(".smart-env"),
            vault_path.join(".smart-connections"),
            vault_path.join(".obsidian").join("plugins").join("smart-connections"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                // Look for embeddings files
                let embeddings_file = candidate.join("embeddings.json");
                let multi_dir = candidate.join("multi");

                if embeddings_file.exists() || multi_dir.exists() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Load embeddings from disk
    pub fn load_embeddings(&mut self) -> Result<usize, SmartConnectionsError> {
        let env_path = self
            .env_path
            .clone()
            .ok_or(SmartConnectionsError::NotAvailable)?;

        self.embeddings.clear();

        // Try to load from embeddings.json
        let embeddings_file = env_path.join("embeddings.json");
        if embeddings_file.exists() {
            self.load_embeddings_json(&embeddings_file)?;
        }

        // Try to load from multi/ directory (newer format)
        let multi_dir = env_path.join("multi");
        if multi_dir.exists() {
            self.load_embeddings_multi(&multi_dir)?;
        }

        self.loaded = true;
        Ok(self.embeddings.len())
    }

    /// Load embeddings from JSON file
    fn load_embeddings_json(&mut self, path: &Path) -> Result<(), SmartConnectionsError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SmartConnectionsError::IoError(e.to_string()))?;

        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SmartConnectionsError::ParseError(e.to_string()))?;

        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                if let Some(embedding) = Self::extract_embedding(value) {
                    self.embeddings.insert(key.clone(), embedding);
                }
            }
        }

        Ok(())
    }

    /// Load embeddings from multi/ directory (JSONL format)
    fn load_embeddings_multi(&mut self, dir: &Path) -> Result<(), SmartConnectionsError> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| SmartConnectionsError::IoError(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl" || e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(key) = data.get("path").and_then(|v| v.as_str()) {
                                if let Some(embedding) = Self::extract_embedding(&data) {
                                    self.embeddings.insert(key.to_string(), embedding);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract embedding vector from a JSON value
    fn extract_embedding(value: &serde_json::Value) -> Option<Vec<f32>> {
        // Try different field names
        let embedding = value.get("embedding")
            .or_else(|| value.get("vec"))
            .or_else(|| value.get("vector"));

        embedding.and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|n| n.as_f64().map(|f| f as f32))
                    .collect()
            })
        })
    }

    /// Find semantically related documents
    pub fn find_related(
        &self,
        query_embedding: &[f32],
        limit: usize,
        min_similarity: f32,
    ) -> Vec<SemanticMatch> {
        if self.embeddings.is_empty() {
            return Vec::new();
        }

        let mut matches: Vec<SemanticMatch> = self
            .embeddings
            .iter()
            .filter_map(|(path, embedding)| {
                let similarity = cosine_similarity(query_embedding, embedding);
                if similarity >= min_similarity {
                    Some(SemanticMatch {
                        path: path.clone(),
                        similarity,
                        excerpt: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity (descending)
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        matches.truncate(limit);
        matches
    }

    /// Find related documents by content (uses simple word overlap if no embeddings)
    pub fn find_related_by_content(
        &self,
        content: &str,
        vault_path: &Path,
        limit: usize,
        min_similarity: f32,
    ) -> Vec<SemanticMatch> {
        // If we have embeddings loaded, we'd need the query embedding
        // For now, fall back to simple keyword matching
        self.find_by_keywords(content, vault_path, limit, min_similarity)
    }

    /// Simple keyword-based search (fallback when embeddings unavailable)
    fn find_by_keywords(
        &self,
        query: &str,
        vault_path: &Path,
        limit: usize,
        min_similarity: f32,
    ) -> Vec<SemanticMatch> {
        let query_words: std::collections::HashSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect();

        if query_words.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();

        // Scan markdown files in vault
        if let Ok(entries) = glob::glob(&format!("{}/**/*.md", vault_path.display())) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(&entry) {
                    let doc_words: std::collections::HashSet<String> = content
                        .to_lowercase()
                        .split_whitespace()
                        .filter(|w| w.len() > 3)
                        .map(|w| w.to_string())
                        .collect();

                    // Jaccard similarity
                    let intersection = query_words.intersection(&doc_words).count();
                    let union = query_words.union(&doc_words).count();

                    if union > 0 {
                        let similarity = intersection as f32 / union as f32;
                        if similarity >= min_similarity {
                            matches.push(SemanticMatch {
                                path: entry.to_string_lossy().to_string(),
                                similarity,
                                excerpt: Self::extract_excerpt(&content, &query_words),
                            });
                        }
                    }
                }
            }
        }

        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(limit);
        matches
    }

    /// Extract a relevant excerpt from content
    fn extract_excerpt(content: &str, keywords: &std::collections::HashSet<String>) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        // Find first line with a keyword match
        for (i, line) in lines.iter().enumerate() {
            let lower = line.to_lowercase();
            if keywords.iter().any(|kw| lower.contains(kw)) {
                // Return context around this line
                let start = i.saturating_sub(1);
                let end = (i + 2).min(lines.len());
                let excerpt: String = lines[start..end].join("\n");

                // Truncate if too long
                if excerpt.len() > 200 {
                    return Some(format!("{}...", &excerpt[..200]));
                }
                return Some(excerpt);
            }
        }

        None
    }

    /// Suggest links for a document based on content
    pub fn suggest_links(
        &self,
        content: &str,
        existing_links: &[String],
        vault_path: &Path,
        limit: usize,
    ) -> Vec<LinkSuggestion> {
        let related = self.find_by_keywords(content, vault_path, limit + existing_links.len(), 0.1);

        related
            .into_iter()
            .filter(|m| {
                // Exclude already linked documents
                !existing_links.iter().any(|link| m.path.contains(link))
            })
            .take(limit)
            .map(|m| LinkSuggestion {
                path: m.path,
                relevance: m.similarity,
                reason: m.excerpt.unwrap_or_else(|| "Content similarity".to_string()),
            })
            .collect()
    }

    /// Detect potential duplicate content
    pub fn detect_duplicates(
        &self,
        content: &str,
        vault_path: &Path,
        threshold: f32,
    ) -> Vec<DuplicateMatch> {
        // Use paragraph-level comparison
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .filter(|p| p.len() > 50)
            .collect();

        if paragraphs.is_empty() {
            return Vec::new();
        }

        let mut duplicates = Vec::new();

        // Scan markdown files
        if let Ok(entries) = glob::glob(&format!("{}/**/*.md", vault_path.display())) {
            for entry in entries.flatten() {
                if let Ok(doc_content) = std::fs::read_to_string(&entry) {
                    for para in &paragraphs {
                        // Simple containment check
                        let para_lower = para.to_lowercase();
                        let doc_lower = doc_content.to_lowercase();

                        // Check for exact or near-exact matches
                        if doc_lower.contains(&para_lower) {
                            duplicates.push(DuplicateMatch {
                                path: entry.to_string_lossy().to_string(),
                                section: para.chars().take(100).collect::<String>() + "...",
                                similarity: 1.0,
                                recommendation: DuplicateRecommendation::Reference,
                            });
                        } else {
                            // Check word overlap similarity
                            let para_words: std::collections::HashSet<_> = para_lower
                                .split_whitespace()
                                .filter(|w| w.len() > 3)
                                .collect();

                            let doc_words: std::collections::HashSet<_> = doc_lower
                                .split_whitespace()
                                .filter(|w| w.len() > 3)
                                .collect();

                            let intersection = para_words.intersection(&doc_words).count();
                            let para_len = para_words.len().max(1);
                            let overlap = intersection as f32 / para_len as f32;

                            if overlap >= threshold {
                                let rec = if overlap > 0.9 {
                                    DuplicateRecommendation::Dedupe
                                } else if overlap > 0.7 {
                                    DuplicateRecommendation::Merge
                                } else {
                                    DuplicateRecommendation::Reference
                                };

                                duplicates.push(DuplicateMatch {
                                    path: entry.to_string_lossy().to_string(),
                                    section: para.chars().take(100).collect::<String>() + "...",
                                    similarity: overlap,
                                    recommendation: rec,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Deduplicate by path and keep highest similarity
        let mut seen: HashMap<String, DuplicateMatch> = HashMap::new();
        for dup in duplicates {
            if let Some(existing) = seen.get(&dup.path) {
                if dup.similarity > existing.similarity {
                    seen.insert(dup.path.clone(), dup);
                }
            } else {
                seen.insert(dup.path.clone(), dup);
            }
        }

        seen.into_values().collect()
    }

    /// Check if embeddings are loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the number of embeddings
    pub fn embedding_count(&self) -> usize {
        self.embeddings.len()
    }
}

impl Default for SmartConnectionsIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// A semantic match result
#[derive(Debug, Clone)]
pub struct SemanticMatch {
    pub path: String,
    pub similarity: f32,
    pub excerpt: Option<String>,
}

/// A link suggestion
#[derive(Debug, Clone)]
pub struct LinkSuggestion {
    pub path: String,
    pub relevance: f32,
    pub reason: String,
}

/// A duplicate content match
#[derive(Debug, Clone)]
pub struct DuplicateMatch {
    pub path: String,
    pub section: String,
    pub similarity: f32,
    pub recommendation: DuplicateRecommendation,
}

/// Recommendation for handling duplicates
#[derive(Debug, Clone)]
pub enum DuplicateRecommendation {
    /// Content is nearly identical - remove one
    Dedupe,
    /// Content is similar - consider merging
    Merge,
    /// Content is related - add reference/link
    Reference,
}

impl std::fmt::Display for DuplicateRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuplicateRecommendation::Dedupe => write!(f, "dedupe"),
            DuplicateRecommendation::Merge => write!(f, "merge"),
            DuplicateRecommendation::Reference => write!(f, "reference"),
        }
    }
}

/// Smart Connections errors
#[derive(Debug)]
pub enum SmartConnectionsError {
    NotAvailable,
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for SmartConnectionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmartConnectionsError::NotAvailable => {
                write!(f, "Smart Connections not available")
            }
            SmartConnectionsError::IoError(msg) => write!(f, "IO error: {}", msg),
            SmartConnectionsError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SmartConnectionsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);

        let a = vec![1.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let expected = 1.0 / 2.0_f32.sqrt();
        assert!((cosine_similarity(&a, &b) - expected).abs() < 0.001);
    }
}
