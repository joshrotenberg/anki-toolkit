//! Tests for model actions.

mod common;

use common::{mock_action, mock_anki_response, setup_mock_server};
use std::collections::HashMap;
use yanki::{AnkiClient, CreateModelParams};

#[tokio::test]
async fn test_model_names() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelNames",
        mock_anki_response(vec!["Basic", "Basic (and reversed card)", "Cloze"]),
    )
    .await;

    let result = client.models().names().await.unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"Basic".to_string()));
}

#[tokio::test]
async fn test_model_names_and_ids() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    let mut expected = HashMap::new();
    expected.insert("Basic", 1234567890_i64);
    expected.insert("Cloze", 9876543210_i64);

    mock_action(&server, "modelNamesAndIds", mock_anki_response(expected)).await;

    let result = client.models().names_and_ids().await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get("Basic"), Some(&1234567890));
}

#[tokio::test]
async fn test_model_field_names() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldNames",
        mock_anki_response(vec!["Front", "Back"]),
    )
    .await;

    let result = client.models().field_names("Basic").await.unwrap();
    assert_eq!(result, vec!["Front", "Back"]);
}

#[tokio::test]
async fn test_create_model() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "createModel",
        mock_anki_response(serde_json::json!({
            "id": 1234567890,
            "name": "My Model"
        })),
    )
    .await;

    let params = CreateModelParams::new("My Model")
        .field("Front")
        .field("Back")
        .css(".card { font-family: arial; }")
        .template("Card 1", "{{Front}}", "{{FrontSide}}<hr>{{Back}}");

    let result = client.models().create(params).await.unwrap();
    assert!(result.get("id").is_some());
}

#[tokio::test]
async fn test_model_styling() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelStyling",
        mock_anki_response(serde_json::json!({
            "css": ".card { font-family: arial; font-size: 20px; }"
        })),
    )
    .await;

    let result = client.models().styling("Basic").await.unwrap();
    assert!(result.css.contains("font-family"));
}

#[tokio::test]
async fn test_model_field_descriptions() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    let mut descriptions = HashMap::new();
    descriptions.insert("Front", "The question");
    descriptions.insert("Back", "The answer");

    mock_action(
        &server,
        "modelFieldDescriptions",
        mock_anki_response(descriptions),
    )
    .await;

    let result = client.models().field_descriptions("Basic").await.unwrap();
    assert_eq!(result.get("Front"), Some(&"The question".to_string()));
}

#[tokio::test]
async fn test_model_field_fonts() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldFonts",
        mock_anki_response(serde_json::json!({
            "Front": {"font": "Arial", "size": 20},
            "Back": {"font": "Arial", "size": 20}
        })),
    )
    .await;

    let result = client.models().field_fonts("Basic").await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get("Front").unwrap().font, "Arial");
}

#[tokio::test]
async fn test_model_fields_on_templates() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldsOnTemplates",
        mock_anki_response(serde_json::json!({
            "Card 1": [["Front"], ["Front", "Back"]]
        })),
    )
    .await;

    let result = client.models().fields_on_templates("Basic").await.unwrap();
    assert!(result.contains_key("Card 1"));
}

#[tokio::test]
async fn test_model_templates() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelTemplates",
        mock_anki_response(serde_json::json!({
            "Card 1": {
                "Front": "{{Front}}",
                "Back": "{{FrontSide}}<hr>{{Back}}"
            }
        })),
    )
    .await;

    let result = client.models().templates("Basic").await.unwrap();
    assert!(result.contains_key("Card 1"));
    let template = result.get("Card 1").unwrap();
    assert_eq!(template.front, "{{Front}}");
}

#[tokio::test]
async fn test_update_styling() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "updateModelStyling",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client
        .models()
        .update_styling("Basic", ".card { font-size: 24px; }")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_templates() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "updateModelTemplates",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let mut templates = HashMap::new();
    templates.insert("Card 1", ("{{Front}}", "{{FrontSide}}<hr>{{Back}}"));

    let result = client.models().update_templates("Basic", templates).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rename_field() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldRename",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client
        .models()
        .rename_field("Basic", "Front", "Question")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reposition_field() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldReposition",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.models().reposition_field("Basic", "Back", 0).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_field() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldAdd",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.models().add_field("Basic", "Extra", Some(2)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_field() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldRemove",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client.models().remove_field("Basic", "Extra").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_field_font() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldSetFont",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client
        .models()
        .set_field_font("Basic", "Front", "Times New Roman")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_field_font_size() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldSetFontSize",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client
        .models()
        .set_field_font_size("Basic", "Front", 24)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_field_description() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(
        &server,
        "modelFieldSetDescription",
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": null,
            "error": null
        })),
    )
    .await;

    let result = client
        .models()
        .set_field_description("Basic", "Front", "Enter the question here")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_and_replace() {
    let server = setup_mock_server().await;
    let client = AnkiClient::builder().url(server.uri()).build();

    mock_action(&server, "findAndReplaceInModels", mock_anki_response(5_i64)).await;

    let params = yanki::FindReplaceParams::new("Basic", "Front", "old", "new");

    let result = client.models().find_and_replace(params).await.unwrap();
    assert_eq!(result, 5);
}
