//! Model-related AnkiConnect actions.
//!
//! This module provides operations for managing note types (models) in Anki.
//!
//! # Example
//!
//! ```no_run
//! use ankit::AnkiClient;
//!
//! # async fn example() -> ankit::Result<()> {
//! let client = AnkiClient::new();
//!
//! // List all models
//! let models = client.models().names().await?;
//! println!("Models: {:?}", models);
//!
//! // Get field names for a model
//! let fields = client.models().field_names("Basic").await?;
//! # Ok(())
//! # }
//! ```

use serde::Serialize;
use std::collections::HashMap;

use crate::client::AnkiClient;
use crate::error::Result;
use crate::types::{
    CardTemplate, CreateModelParams, FieldFont, FieldsOnTemplates, FindReplaceParams, ModelStyling,
};

/// Provides access to model-related AnkiConnect operations.
///
/// Obtained via [`AnkiClient::models()`].
#[derive(Debug)]
pub struct ModelActions<'a> {
    pub(crate) client: &'a AnkiClient,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelNameParams<'a> {
    model_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameFieldParams<'a> {
    model_name: &'a str,
    old_field_name: &'a str,
    new_field_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositionFieldParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
    index: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddFieldParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoveFieldParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetFieldFontParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
    font: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetFieldSizeParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
    font_size: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetFieldDescriptionParams<'a> {
    model_name: &'a str,
    field_name: &'a str,
    description: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FindModelsByIdParams<'a> {
    model_ids: &'a [i64],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FindModelsByNameParams<'a> {
    model_names: &'a [&'a str],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateRenameParams<'a> {
    model_name: &'a str,
    old_template_name: &'a str,
    new_template_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateRepositionParams<'a> {
    model_name: &'a str,
    template_name: &'a str,
    index: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateAddParams<'a> {
    model_name: &'a str,
    template: TemplateAddInner<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateAddInner<'a> {
    #[serde(rename = "Name")]
    name: &'a str,
    #[serde(rename = "Front")]
    front: &'a str,
    #[serde(rename = "Back")]
    back: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateRemoveParams<'a> {
    model_name: &'a str,
    template_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStylingParams<'a> {
    model: UpdateStylingModel<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStylingModel<'a> {
    name: &'a str,
    css: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTemplatesParams<'a> {
    model: UpdateTemplatesModel<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTemplatesModel<'a> {
    name: &'a str,
    templates: HashMap<&'a str, TemplateContent<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateContent<'a> {
    #[serde(rename = "Front")]
    front: &'a str,
    #[serde(rename = "Back")]
    back: &'a str,
}

impl<'a> ModelActions<'a> {
    /// Get all model names.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let models = client.models().names().await?;
    /// println!("Available models: {:?}", models);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn names(&self) -> Result<Vec<String>> {
        self.client.invoke_without_params("modelNames").await
    }

    /// Get all model names and their IDs.
    ///
    /// Returns a map of model name to model ID.
    pub async fn names_and_ids(&self) -> Result<HashMap<String, i64>> {
        self.client.invoke_without_params("modelNamesAndIds").await
    }

    /// Get field names for a model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let fields = client.models().field_names("Basic").await?;
    /// // ["Front", "Back"]
    /// # Ok(())
    /// # }
    /// ```
    pub async fn field_names(&self, model_name: &str) -> Result<Vec<String>> {
        self.client
            .invoke("modelFieldNames", ModelNameParams { model_name })
            .await
    }

    /// Get field descriptions for a model.
    pub async fn field_descriptions(&self, model_name: &str) -> Result<HashMap<String, String>> {
        self.client
            .invoke("modelFieldDescriptions", ModelNameParams { model_name })
            .await
    }

    /// Get font settings for all fields in a model.
    pub async fn field_fonts(&self, model_name: &str) -> Result<HashMap<String, FieldFont>> {
        self.client
            .invoke("modelFieldFonts", ModelNameParams { model_name })
            .await
    }

    /// Get fields used on each template side.
    ///
    /// Returns a map of template name to [front_fields, back_fields].
    pub async fn fields_on_templates(&self, model_name: &str) -> Result<FieldsOnTemplates> {
        self.client
            .invoke("modelFieldsOnTemplates", ModelNameParams { model_name })
            .await
    }

    /// Create a new model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ankit::{AnkiClient, CreateModelParams};
    ///
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let params = CreateModelParams::new("My Model")
    ///     .field("Front")
    ///     .field("Back")
    ///     .css(".card { font-family: arial; }")
    ///     .template("Card 1", "{{Front}}", "{{FrontSide}}<hr>{{Back}}");
    ///
    /// let model = client.models().create(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, params: CreateModelParams) -> Result<serde_json::Value> {
        self.client.invoke("createModel", params).await
    }

    /// Get card templates for a model.
    pub async fn templates(&self, model_name: &str) -> Result<HashMap<String, CardTemplate>> {
        self.client
            .invoke("modelTemplates", ModelNameParams { model_name })
            .await
    }

    /// Get styling (CSS) for a model.
    pub async fn styling(&self, model_name: &str) -> Result<ModelStyling> {
        self.client
            .invoke("modelStyling", ModelNameParams { model_name })
            .await
    }

    /// Update card templates for a model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # use std::collections::HashMap;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    ///
    /// let mut templates = HashMap::new();
    /// templates.insert("Card 1", ("{{Front}}", "{{FrontSide}}<hr>{{Back}}"));
    ///
    /// client.models().update_templates("Basic", templates).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_templates(
        &self,
        model_name: &str,
        templates: HashMap<&str, (&str, &str)>,
    ) -> Result<()> {
        let template_map: HashMap<&str, TemplateContent> = templates
            .into_iter()
            .map(|(name, (front, back))| (name, TemplateContent { front, back }))
            .collect();

        self.client
            .invoke_void(
                "updateModelTemplates",
                UpdateTemplatesParams {
                    model: UpdateTemplatesModel {
                        name: model_name,
                        templates: template_map,
                    },
                },
            )
            .await
    }

    /// Update styling (CSS) for a model.
    pub async fn update_styling(&self, model_name: &str, css: &str) -> Result<()> {
        self.client
            .invoke_void(
                "updateModelStyling",
                UpdateStylingParams {
                    model: UpdateStylingModel {
                        name: model_name,
                        css,
                    },
                },
            )
            .await
    }

    /// Find and replace text in model fields.
    pub async fn find_and_replace(&self, params: FindReplaceParams) -> Result<i64> {
        self.client.invoke("findAndReplaceInModels", params).await
    }

    /// Rename a field in a model.
    pub async fn rename_field(
        &self,
        model_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldRename",
                RenameFieldParams {
                    model_name,
                    old_field_name: old_name,
                    new_field_name: new_name,
                },
            )
            .await
    }

    /// Reposition a field in a model.
    ///
    /// The index is 0-based.
    pub async fn reposition_field(
        &self,
        model_name: &str,
        field_name: &str,
        index: i32,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldReposition",
                RepositionFieldParams {
                    model_name,
                    field_name,
                    index,
                },
            )
            .await
    }

    /// Add a new field to a model.
    ///
    /// If index is None, the field is added at the end.
    pub async fn add_field(
        &self,
        model_name: &str,
        field_name: &str,
        index: Option<i32>,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldAdd",
                AddFieldParams {
                    model_name,
                    field_name,
                    index,
                },
            )
            .await
    }

    /// Remove a field from a model.
    pub async fn remove_field(&self, model_name: &str, field_name: &str) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldRemove",
                RemoveFieldParams {
                    model_name,
                    field_name,
                },
            )
            .await
    }

    /// Set the font for a field.
    pub async fn set_field_font(
        &self,
        model_name: &str,
        field_name: &str,
        font: &str,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldSetFont",
                SetFieldFontParams {
                    model_name,
                    field_name,
                    font,
                },
            )
            .await
    }

    /// Set the font size for a field.
    pub async fn set_field_font_size(
        &self,
        model_name: &str,
        field_name: &str,
        size: i32,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldSetFontSize",
                SetFieldSizeParams {
                    model_name,
                    field_name,
                    font_size: size,
                },
            )
            .await
    }

    /// Set the description for a field.
    pub async fn set_field_description(
        &self,
        model_name: &str,
        field_name: &str,
        description: &str,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelFieldSetDescription",
                SetFieldDescriptionParams {
                    model_name,
                    field_name,
                    description,
                },
            )
            .await
    }

    /// Find models by their IDs.
    ///
    /// Returns full model information for each ID.
    pub async fn find_by_id(&self, model_ids: &[i64]) -> Result<Vec<serde_json::Value>> {
        self.client
            .invoke("findModelsById", FindModelsByIdParams { model_ids })
            .await
    }

    /// Find models by name patterns.
    ///
    /// Supports wildcards: `*` matches any sequence, `_` matches any single character.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// let models = client.models().find_by_name(&["Basic*"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_by_name(&self, model_names: &[&str]) -> Result<Vec<serde_json::Value>> {
        self.client
            .invoke("findModelsByName", FindModelsByNameParams { model_names })
            .await
    }

    /// Rename a card template.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.models().rename_template("Basic", "Card 1", "Front to Back").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename_template(
        &self,
        model_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelTemplateRename",
                TemplateRenameParams {
                    model_name,
                    old_template_name: old_name,
                    new_template_name: new_name,
                },
            )
            .await
    }

    /// Reposition a card template within a model.
    ///
    /// The index is 0-based.
    pub async fn reposition_template(
        &self,
        model_name: &str,
        template_name: &str,
        index: i32,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelTemplateReposition",
                TemplateRepositionParams {
                    model_name,
                    template_name,
                    index,
                },
            )
            .await
    }

    /// Add a new card template to a model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ankit::AnkiClient;
    /// # async fn example() -> ankit::Result<()> {
    /// let client = AnkiClient::new();
    /// client.models().add_template(
    ///     "Basic",
    ///     "Reverse",
    ///     "{{Back}}",
    ///     "{{FrontSide}}<hr>{{Front}}"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_template(
        &self,
        model_name: &str,
        template_name: &str,
        front: &str,
        back: &str,
    ) -> Result<()> {
        self.client
            .invoke_void(
                "modelTemplateAdd",
                TemplateAddParams {
                    model_name,
                    template: TemplateAddInner {
                        name: template_name,
                        front,
                        back,
                    },
                },
            )
            .await
    }

    /// Remove a card template from a model.
    ///
    /// Note: A model must have at least one template.
    pub async fn remove_template(&self, model_name: &str, template_name: &str) -> Result<()> {
        self.client
            .invoke_void(
                "modelTemplateRemove",
                TemplateRemoveParams {
                    model_name,
                    template_name,
                },
            )
            .await
    }
}
