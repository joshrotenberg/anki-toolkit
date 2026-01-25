//! Model-related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a model (note type).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Model name.
    pub name: String,
    /// Model ID.
    #[serde(rename = "id")]
    pub model_id: i64,
}

/// Field information for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelField {
    /// Field name.
    pub name: String,
    /// Field ordinal position.
    #[serde(default)]
    pub ord: i32,
    /// Whether this is a sticky field.
    #[serde(default)]
    pub sticky: bool,
    /// Whether to use right-to-left text.
    #[serde(default)]
    pub rtl: bool,
    /// Font name for editing.
    #[serde(default)]
    pub font: String,
    /// Font size for editing.
    #[serde(default)]
    pub size: i32,
    /// Field description.
    #[serde(default)]
    pub description: String,
}

/// Card template for a model (used in responses from modelTemplates).
///
/// Note: When retrieved via `modelTemplates`, the template name is the HashMap key,
/// not a field in this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardTemplate {
    /// Front template HTML.
    #[serde(rename = "Front")]
    pub front: String,
    /// Back template HTML.
    #[serde(rename = "Back")]
    pub back: String,
}

/// Card template for creating a model (includes name).
#[derive(Debug, Clone, Serialize)]
pub struct CreateCardTemplate {
    /// Template name.
    #[serde(rename = "Name")]
    pub name: String,
    /// Front template HTML.
    #[serde(rename = "Front")]
    pub front: String,
    /// Back template HTML.
    #[serde(rename = "Back")]
    pub back: String,
}

/// Font information for a field.
#[derive(Debug, Clone, Deserialize)]
pub struct FieldFont {
    /// Font name.
    pub font: String,
    /// Font size.
    pub size: i32,
}

/// Parameters for creating a new model.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateModelParams {
    /// Model name.
    pub model_name: String,
    /// Field names for the model.
    pub in_order_fields: Vec<String>,
    /// CSS styling for the model.
    pub css: String,
    /// Whether this is a cloze model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cloze: Option<bool>,
    /// Card templates.
    pub card_templates: Vec<CreateCardTemplate>,
}

impl CreateModelParams {
    /// Create new model parameters.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            model_name: name.into(),
            in_order_fields: Vec::new(),
            css: String::new(),
            is_cloze: None,
            card_templates: Vec::new(),
        }
    }

    /// Add a field to the model.
    pub fn field(mut self, name: impl Into<String>) -> Self {
        self.in_order_fields.push(name.into());
        self
    }

    /// Set the CSS styling.
    pub fn css(mut self, css: impl Into<String>) -> Self {
        self.css = css.into();
        self
    }

    /// Set whether this is a cloze model.
    pub fn cloze(mut self, is_cloze: bool) -> Self {
        self.is_cloze = Some(is_cloze);
        self
    }

    /// Add a card template.
    pub fn template(
        mut self,
        name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.card_templates.push(CreateCardTemplate {
            name: name.into(),
            front: front.into(),
            back: back.into(),
        });
        self
    }
}

/// Parameters for find and replace in models.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindReplaceParams {
    /// Notes to search in (empty for all notes).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<i64>,
    /// Action (usually "findAndReplaceInModels").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Model name.
    pub model_name: String,
    /// Field name.
    pub field_name: String,
    /// Search pattern.
    pub find_text: String,
    /// Replacement text.
    pub replace_text: String,
    /// Use regex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<bool>,
    /// Match case.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_case: Option<bool>,
}

impl FindReplaceParams {
    /// Create new find and replace parameters.
    pub fn new(
        model_name: impl Into<String>,
        field_name: impl Into<String>,
        find: impl Into<String>,
        replace: impl Into<String>,
    ) -> Self {
        Self {
            notes: Vec::new(),
            action: None,
            model_name: model_name.into(),
            field_name: field_name.into(),
            find_text: find.into(),
            replace_text: replace.into(),
            regex: None,
            match_case: None,
        }
    }

    /// Limit to specific notes.
    pub fn notes(mut self, notes: Vec<i64>) -> Self {
        self.notes = notes;
        self
    }

    /// Enable regex matching.
    pub fn regex(mut self, enabled: bool) -> Self {
        self.regex = Some(enabled);
        self
    }

    /// Enable case-sensitive matching.
    pub fn match_case(mut self, enabled: bool) -> Self {
        self.match_case = Some(enabled);
        self
    }
}

/// Model styling information.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelStyling {
    /// CSS styling.
    pub css: String,
}

/// Fields on templates response.
pub type FieldsOnTemplates = HashMap<String, Vec<Vec<String>>>;
