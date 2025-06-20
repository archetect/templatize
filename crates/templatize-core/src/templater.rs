use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tracing::debug;
use regex::Regex;
use convert_case::{Case, Casing};

pub struct ExactTemplater {
    token: String,
    replacement: String,
}

pub struct JinjaEscaper {
    jinja_pattern: Regex,
}

pub struct CaseShapeTemplater {
    replacements: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CaseShapeMapping {
    pub original: String,
    pub replacement: String,
}

pub struct TemplateOptions {
    pub process_paths: bool,
    pub process_contents: bool,
    pub dry_run: bool,
}

impl ExactTemplater {
    pub fn new(token: &str, replacement: &str) -> Self {
        Self {
            token: token.to_string(),
            replacement: replacement.to_string(),
        }
    }

    pub fn process_content(&self, content: &str) -> Option<String> {
        if content.contains(&self.token) {
            let new_content = content.replace(&self.token, &self.replacement);
            debug!("Content replacement: found {} occurrences", content.matches(&self.token).count());
            Some(new_content)
        } else {
            None
        }
    }

    pub fn process_path_component(&self, path: &Path) -> Option<String> {
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.contains(&self.token) {
                    let new_name = name_str.replace(&self.token, &self.replacement);
                    debug!("Path replacement: '{}' -> '{}'", name_str, new_name);
                    return Some(new_name);
                }
            }
        }
        None
    }

    pub fn process_full_path(&self, path: &Path) -> Option<PathBuf> {
        // Convert path to string for replacement
        if let Some(path_str) = path.to_str() {
            // Normalize path separators to forward slashes for consistent matching
            let normalized_path = path_str.replace('\\', "/");
            let normalized_token = self.token.replace('\\', "/");
            
            if normalized_path.contains(&normalized_token) {
                let new_path_str = normalized_path.replace(&normalized_token, &self.replacement);
                debug!("Full path replacement: '{}' -> '{}'", path_str, new_path_str);
                
                // Convert back to PathBuf with proper separators for the current OS
                return Some(PathBuf::from(new_path_str));
            }
        }
        None
    }
}

impl JinjaEscaper {
    pub fn new() -> Result<Self, regex::Error> {
        let jinja_pattern = Regex::new(r"\{\{\s*([^}]+)\s*\}\}")?;
        Ok(Self { jinja_pattern })
    }

    pub fn escape_content(&self, content: &str) -> Option<String> {
        if self.jinja_pattern.is_match(content) {
            let escaped = self.jinja_pattern.replace_all(content, |caps: &regex::Captures| {
                let inner = caps.get(1).unwrap().as_str().trim();
                format!("{{{{'{{'}}}}{{ {} }}", inner)
            });
            let count = self.jinja_pattern.find_iter(content).count();
            debug!("Jinja escaping: found {} Jinja expressions", count);
            Some(escaped.to_string())
        } else {
            None
        }
    }
}

impl CaseShapeTemplater {
    pub fn new(token: &str, replacement: &str) -> Result<Self, anyhow::Error> {
        // Validate that both token and replacement are compound words
        Self::validate_compound_word(token, "token")?;
        Self::validate_compound_word(replacement, "replacement")?;

        let mut replacements = HashMap::new();
        
        // Generate all case shape variants
        let cases = [
            Case::Camel,      // camelCase
            Case::Pascal,     // PascalCase  
            Case::Kebab,      // kebab-case
            Case::Snake,      // snake_case
            Case::Train,      // Train-Case
            Case::ScreamingSnake, // SCREAMING_SNAKE_CASE
            Case::Cobol,      // COBOL-CASE
        ];

        for case in &cases {
            let token_variant = token.to_case(*case);
            
            // Extract and convert the variable content from the replacement template
            let replacement_variant = if replacement.contains("{{") && replacement.contains("}}") {
                let jinja_pattern = Regex::new(r"\{\{\s*([^}]+)\s*\}\}").unwrap();
                if let Some(caps) = jinja_pattern.captures(&replacement) {
                    let inner_content = caps.get(1).unwrap().as_str().trim();
                    let converted_inner = inner_content.to_case(*case);
                    format!("{{{{ {} }}}}", converted_inner)
                } else {
                    replacement.to_case(*case)
                }
            } else {
                replacement.to_case(*case)
            };
            
            debug!("Case shape mapping: {} -> {}", token_variant, replacement_variant);
            replacements.insert(token_variant, replacement_variant);
        }

        // Also include the original forms
        replacements.insert(token.to_string(), replacement.to_string());

        Ok(Self { replacements })
    }

    fn validate_compound_word(word: &str, field_name: &str) -> Result<(), anyhow::Error> {
        // Remove Jinja syntax for validation if present
        let clean_word = if word.contains("{{") && word.contains("}}") {
            // Extract content between {{ }}
            let jinja_pattern = Regex::new(r"\{\{\s*([^}]+)\s*\}\}").unwrap();
            if let Some(caps) = jinja_pattern.captures(word) {
                caps.get(1).unwrap().as_str().trim()
            } else {
                word
            }
        } else {
            word
        };

        // Check if word contains separators indicating compound nature
        let has_separators = clean_word.contains('-') || 
                           clean_word.contains('_') || 
                           clean_word.chars().any(|c| c.is_uppercase());

        if !has_separators {
            anyhow::bail!(
                "The {} '{}' does not appear to be a compound word. \
                Compound words should contain separators like hyphens (-), underscores (_), \
                or mixed case (e.g., 'example-name', 'project_name', 'ProjectName')",
                field_name, word
            );
        }

        Ok(())
    }

    pub fn get_mappings(&self) -> Vec<CaseShapeMapping> {
        self.replacements.iter()
            .map(|(k, v)| CaseShapeMapping {
                original: k.clone(),
                replacement: v.clone(),
            })
            .collect()
    }

    pub fn process_content(&self, content: &str) -> Option<String> {
        let mut modified_content = content.to_string();
        let mut found_replacements = false;

        // Sort by length (longest first) to avoid partial matches
        let mut sorted_replacements: Vec<_> = self.replacements.iter().collect();
        sorted_replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (token, replacement) in sorted_replacements {
            if modified_content.contains(token) {
                modified_content = modified_content.replace(token, replacement);
                found_replacements = true;
                debug!("Case shape replacement: '{}' -> '{}'", token, replacement);
            }
        }

        if found_replacements {
            Some(modified_content)
        } else {
            None
        }
    }

    pub fn process_path_component(&self, path: &Path) -> Option<String> {
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if let Some(new_content) = self.process_content(name_str) {
                    if new_content != name_str {
                        debug!("Case shape path replacement: '{}' -> '{}'", name_str, new_content);
                        return Some(new_content);
                    }
                }
            }
        }
        None
    }

    pub fn process_full_path(&self, path: &Path) -> Option<PathBuf> {
        // Convert path to string for replacement
        if let Some(path_str) = path.to_str() {
            // Normalize path separators to forward slashes for consistent matching
            let normalized_path = path_str.replace('\\', "/");
            
            if let Some(new_path_content) = self.process_content(&normalized_path) {
                if new_path_content != normalized_path {
                    debug!("Case shape full path replacement: '{}' -> '{}'", path_str, new_path_content);
                    // Convert back to PathBuf with proper separators for the current OS
                    return Some(PathBuf::from(new_path_content));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_replacement() {
        let templater = ExactTemplater::new("example-name", "{{ project-name }}");
        
        let content = "This is an example-name project with example-name references.";
        let result = templater.process_content(content);
        
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "This is an {{ project-name }} project with {{ project-name }} references."
        );
    }

    #[test]
    fn test_no_content_replacement() {
        let templater = ExactTemplater::new("example-name", "{{ project-name }}");
        
        let content = "This is a test project with no matching tokens.";
        let result = templater.process_content(content);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_path_replacement() {
        let templater = ExactTemplater::new("example-name", "{{ project-name }}");
        
        let path = Path::new("/some/path/example-name-file.txt");
        let result = templater.process_path_component(path);
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "{{ project-name }}-file.txt");
    }

    #[test]
    fn test_no_path_replacement() {
        let templater = ExactTemplater::new("example-name", "{{ project-name }}");
        
        let path = Path::new("/some/path/other-file.txt");
        let result = templater.process_path_component(path);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_jinja_escaping() {
        let escaper = JinjaEscaper::new().unwrap();
        
        let content = "This {{ project-name }} has {{ some-value }} and {{another-var}}.";
        let result = escaper.escape_content(content);
        
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "This {{'{'}}{ project-name } has {{'{'}}{ some-value } and {{'{'}}{ another-var }."
        );
    }

    #[test]
    fn test_no_jinja_escaping() {
        let escaper = JinjaEscaper::new().unwrap();
        
        let content = "This content has no Jinja syntax to escape.";
        let result = escaper.escape_content(content);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_jinja_escaping_with_spaces() {
        let escaper = JinjaEscaper::new().unwrap();
        
        let content = "{{ project-name }} and {{  spaced-var  }} should both be escaped.";
        let result = escaper.escape_content(content);
        
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "{{'{'}}{ project-name } and {{'{'}}{ spaced-var } should both be escaped."
        );
    }

    #[test]
    fn test_case_shape_templater_creation() {
        let templater = CaseShapeTemplater::new("example-name", "{{ project-name }}").unwrap();
        let mappings = templater.get_mappings();
        
        
        // Should have 7 case variants (original is same as kebab-case so gets deduplicated)
        assert_eq!(mappings.len(), 7);
        
        // Check some specific mappings
        let mapping_map: std::collections::HashMap<String, String> = 
            mappings.iter().map(|m| (m.original.clone(), m.replacement.clone())).collect();
        
        assert_eq!(mapping_map.get("exampleName"), Some(&"{{ projectName }}".to_string()));
        assert_eq!(mapping_map.get("ExampleName"), Some(&"{{ ProjectName }}".to_string()));
        assert_eq!(mapping_map.get("EXAMPLE_NAME"), Some(&"{{ PROJECT_NAME }}".to_string()));
        assert_eq!(mapping_map.get("example-name"), Some(&"{{ project-name }}".to_string()));
    }

    #[test]
    fn test_case_shape_content_replacement() {
        let templater = CaseShapeTemplater::new("example-name", "{{ project-name }}").unwrap();
        
        let content = "final ExampleName exampleName = new ExampleName();";
        let result = templater.process_content(content);
        
        
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "final {{ ProjectName }} {{ projectName }} = new {{ ProjectName }}();"
        );
    }

    #[test]
    fn test_case_shape_validation_failure() {
        // Should fail with single word
        let result = CaseShapeTemplater::new("example", "{{ project }}");
        assert!(result.is_err());
        
        // Should fail with no separators
        let result = CaseShapeTemplater::new("exampleproject", "{{ projectname }}");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_shape_validation_success() {
        // Should succeed with various compound word formats
        assert!(CaseShapeTemplater::new("example-name", "{{ project-name }}").is_ok());
        assert!(CaseShapeTemplater::new("example_name", "{{ project_name }}").is_ok());
        assert!(CaseShapeTemplater::new("ExampleName", "{{ ProjectName }}").is_ok());
        assert!(CaseShapeTemplater::new("exampleName", "{{ projectName }}").is_ok());
    }

    #[test]
    fn test_exact_full_path_replacement() {
        let templater = ExactTemplater::new("com/acme/widgets", "{{ package-root }}");
        
        let path = Path::new("src/main/java/com/acme/widgets/entities/User.java");
        let result = templater.process_full_path(path);
        
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().to_str().unwrap(),
            "src/main/java/{{ package-root }}/entities/User.java"
        );
    }

    #[test]
    fn test_exact_full_path_no_replacement() {
        let templater = ExactTemplater::new("com/acme/widgets", "{{ package-root }}");
        
        let path = Path::new("src/main/java/com/other/package/Other.java");
        let result = templater.process_full_path(path);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_case_shape_full_path_replacement() {
        let templater = CaseShapeTemplater::new("acme-widgets", "{{ package-name }}").unwrap();
        
        let path = Path::new("src/main/java/acme-widgets/entities/AcmeWidgets.java");
        let result = templater.process_full_path(path);
        
        assert!(result.is_some());
        let result_path = result.unwrap();
        let result_str = result_path.to_str().unwrap();
        // Should replace both the path component and the file name component with appropriate case variants
        assert!(result_str.contains("{{ package-name }}"));
        assert!(result_str.contains("{{ PackageName }}"));
    }

    #[test]
    fn test_windows_path_normalization() {
        let templater = ExactTemplater::new("com/acme/widgets", "{{ package-root }}");
        
        // Test with Windows-style paths
        let path = Path::new("src\\main\\java\\com\\acme\\widgets\\entities\\User.java");
        let result = templater.process_full_path(path);
        
        assert!(result.is_some());
        let result_path = result.unwrap();
        let result_str = result_path.to_str().unwrap();
        assert!(result_str.contains("{{ package-root }}"));
    }
}