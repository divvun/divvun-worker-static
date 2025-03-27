use std::collections::HashMap;

use clap::Parser;
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::Cors,
    web::{Data, Html, Json},
    EndpointExt, IntoResponse, Route, Server,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Available {
    grammar: HashMap<String, String>,
    speller: HashMap<String, String>,
    hyphenation: HashMap<String, String>,
    tts: HashMap<String, TtsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TtsConfig {
    name: String,
    voices: HashMap<String, VoiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoiceConfig {
    name: String,
    gender: String,
    #[serde(default)]
    speaker: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Languages {
    available: Available,
}

#[handler]
async fn languages_get(Data(languages): Data<&Languages>) -> impl IntoResponse {
    Json(languages).into_response()
}

#[handler]
async fn health_get() -> impl IntoResponse {
    Json(json!({ "status": "ok" })).into_response()
}

#[handler]
async fn index_get(Data(languages): Data<&Languages>) -> impl IntoResponse {
    let mut html = include_str!("../index.html").to_string();
    
    // Find the position to insert the generated sections
    if let Some(pos) = html.find("<h2>Endpoints</h2>") {
        let insert_pos = html[pos..].find("</section>").unwrap_or(0) + pos;
        
        let mut sections = Vec::new();

        // Grammar section
        if !languages.available.grammar.is_empty() {
            let mut sorted_langs: Vec<_> = languages.available.grammar.iter().collect();
            sorted_langs.sort_by_key(|(tag, _)| *tag);
            
            sections.push(format!(
                r#"            <div class="endpoint" id="grammar">
                <h3>Grammar Check</h3>
                <p><span class="method post">POST</span> <code>/grammar/:tag</code> <span class="response-type">application/json</span></p>
                <p>Check grammar for text. Available languages:</p>
                <ul>
{}
                </ul>
                <details>
                    <summary>Request</summary>
                    <pre><code>{{
    "text": "sami"
}}</code></pre>
                </details>
                <details>
                    <summary>Response</summary>
                    <pre><code>{{
  "text": "sami",
  "errs": [
    {{
      "error_text": "sami",
      "start_index": 0,
      "end_index": 4,
      "error_code": "typo",
      "description": "Ii leat sátnelisttus",
      "suggestions": [
        "sámi"
      ],
      "title": "Čállinmeattáhus"
    }}
  ]
}}</code></pre>
                </details>
            </div>"#,
                sorted_langs.iter()
                    .map(|(tag, name)| format!(
                        "                <li><a href=\"/grammar/{}\"><code>{}</code></a> - {}</li>",
                        tag, tag, name
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Speller section
        if !languages.available.speller.is_empty() {
            let mut sorted_langs: Vec<_> = languages.available.speller.iter().collect();
            sorted_langs.sort_by_key(|(tag, _)| *tag);
            
            sections.push(format!(
                r#"            <div class="endpoint" id="speller">
                <h3>Spell Check</h3>
                <p><span class="method post">POST</span> <code>/speller/:tag</code> <span class="response-type">application/json</span></p>
                <p>Check spelling for text. Available languages:</p>
                <ul>
{}
                </ul>
                <details>
                    <summary>Request</summary>
                    <pre><code>{{
    "text": "sami"
}}</code></pre>
                </details>
                <details>
                    <summary>Response</summary>
                    <pre><code>{{
  "text": "sami",
  "results": [
    {{
      "word": "sami",
      "is_correct": false,
      "suggestions": [
        {{
          "value": "sámi",
          "weight": 14.529631
        }},
        {{
          "value": "sama",
          "weight": 40.2973
        }},
        {{
          "value": "sáme",
          "weight": 45.896103
        }},
        {{
          "value": "sabmi",
          "weight": 50.2973
        }},
        {{
          "value": "samai",
          "weight": 50.2973
        }},
        {{
          "value": "sapmi",
          "weight": 50.2973
        }},
        {{
          "value": "satmi",
          "weight": 50.2973
        }},
        {{
          "value": "samo",
          "weight": 55.2973
        }},
        {{
          "value": "samu",
          "weight": 55.2973
        }},
        {{
          "value": "somá",
          "weight": 56.623154
        }}
      ]
    }}
  ]
}}</code></pre>
                </details>
            </div>"#,
                sorted_langs.iter()
                    .map(|(tag, name)| format!(
                        "                <li><a href=\"/speller/{}\"><code>{}</code></a> - {}</li>",
                        tag, tag, name
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // TTS section
        if !languages.available.tts.is_empty() {
            let mut sorted_langs: Vec<_> = languages.available.tts.iter().collect();
            sorted_langs.sort_by_key(|(tag, _)| *tag);
            
            sections.push(format!(
                r#"            <div class="endpoint" id="tts">
                <h3>Text-to-Speech</h3>
                <p><span class="method post">POST</span> <code>/tts/:tag/:voice</code> <span class="response-type">audio/wav</span></p>
                <p>Convert text to speech. Available languages and voices:</p>
                <ul>
{}
                </ul>
                <details>
                    <summary>Request</summary>
                    <pre><code>{{
    "text": "Sample text to convert to speech"
}}</code></pre>
                </details>
                <details>
                    <summary>Response</summary>
                    <p>WAV audio file containing the synthesized speech.</p>
                </details>
            </div>"#,
                sorted_langs.iter()
                    .map(|(tag, config)| {
                        let voices = config.voices.iter()
                            .map(|(voice_id, voice)| {
                                let gender_icon = if voice.gender == "female" { "♀" } else { "♂" };
                                format!(
                                    "<code>{}</code> <a href=\"/tts/{}/{}\">{} {}</a>",
                                    voice_id, tag, voice_id, voice.name, gender_icon
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "                <li><code>{}</code> - {} (voices: {})</li>",
                            tag, config.name, voices
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        
        html.insert_str(insert_pos, &format!("\n{}\n", sections.join("\n\n")));
    }

    Html(html).into_response()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to run the server on
    #[arg(long, default_value_t = 4000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    Ok(run(cli).await?)
}

const LANGUAGES: &str = include_str!("../languages.toml");

async fn run(cli: Cli) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Parse languages from TOML
    let languages: Languages = toml::from_str(LANGUAGES)?;

    let app = Route::new()
        .at("/", get(index_get))
        .at("/health", get(health_get))
        .at("/languages", get(languages_get))
        .data(languages)
        .with(Cors::default());

    Server::new(TcpListener::bind((cli.host, cli.port)))
        .run(app)
        .await?;

    Ok(())
}
