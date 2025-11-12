use std::collections::HashMap;
use std::fs;

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
struct LanguagesConfig {
    config: Config,
    grammar: HashMap<String, ServiceConfig>,
    speller: HashMap<String, ServiceConfig>,
    hyphenation: HashMap<String, ServiceConfig>,
    tts: HashMap<String, TtsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceConfig {
    name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyLanguagesConfig {
    grammar: HashMap<String, String>,
    speller: HashMap<String, String>,
    hyphenation: HashMap<String, String>,
}

impl From<&LanguagesConfig> for LegacyLanguagesConfig {
    fn from(languages: &LanguagesConfig) -> Self {
        Self {
            grammar: languages
                .grammar
                .iter()
                .map(|(k, v)| (k.clone(), v.name.clone()))
                .collect(),
            speller: languages
                .speller
                .iter()
                .map(|(k, v)| (k.clone(), v.name.clone()))
                .collect(),
            hyphenation: languages
                .hyphenation
                .iter()
                .map(|(k, v)| (k.clone(), v.name.clone()))
                .collect(),
        }
    }
}

#[handler]
async fn languages_get(Data(languages): Data<&LanguagesConfig>) -> impl IntoResponse {
    Json(serde_json::json!({ "available": LegacyLanguagesConfig::from(languages) })).into_response()
}

#[handler]
async fn health_get() -> impl IntoResponse {
    Json(json!({ "status": "ok" })).into_response()
}

#[handler]
async fn index_get(Data(languages): Data<&LanguagesConfig>) -> impl IntoResponse {
    let mut html = include_str!("../index.html").to_string();

    // Replace base URL placeholder
    html = html.replace("{{BASE_URL}}", &languages.config.base_url);

    // Find the position to insert the generated sections
    if let Some(pos) = html.find("<h2>Endpoints</h2>") {
        let insert_pos = html[pos..].find("</section>").unwrap_or(0) + pos;

        let mut sections = Vec::new();

        // Grammar section
        if !languages.grammar.is_empty() {
            let mut sorted_langs: Vec<_> = languages.grammar.iter().collect();
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
                    <summary>Request <code>application/json</code></summary>
                    <pre><code>{{
    "text": "sami"
}}</code></pre>
                </details>
                <details>
                    <summary>Response <code>application/json</code></summary>
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
                    .map(|(tag, service)| format!(
                        "                <li><a href=\"/grammar/{}\"><code>{}</code></a> - {}</li>",
                        tag, tag, service.name
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Speller section
        if !languages.speller.is_empty() {
            let mut sorted_langs: Vec<_> = languages.speller.iter().collect();
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
                    <summary>Request <code>application/json</code></summary>
                    <pre><code>{{
    "text": "sami"
}}</code></pre>
                </details>
                <details>
                    <summary>Response <code>application/json</code></summary>
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
                    .map(|(tag, service)| format!(
                        "                <li><a href=\"/speller/{}\"><code>{}</code></a> - {}</li>",
                        tag, tag, service.name
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // TTS section
        if !languages.tts.is_empty() {
            let mut sorted_langs: Vec<_> = languages.tts.iter().collect();
            sorted_langs.sort_by_key(|(tag, _)| *tag);

            sections.push(format!(
                r#"            <div class="endpoint" id="tts">
                <h3>Text-to-Speech</h3>
                <p><span class="method post">POST</span> <code>/tts/:tag/:voice</code> <span class="response-type">audio/wav</span></p>
                <p><strong>MP3:</strong> add <code>Accept: audio/mpeg</code> header to get MP3 audio instead of WAV.</p>
                <p>Convert text to speech. Available languages and voices:</p>
                <ul>
{}
                </ul>
                <details>
                    <summary>Request <code>application/json</code></summary>
                    <pre><code>{{
    "text": "Sample text to convert to speech"
}}</code></pre>
                </details>
                <details>
                    <summary>Response <code>audio/wav</code></summary>
                    <p>WAV audio file containing the synthesized speech.</p>
                </details>
                <details>
                    <summary>Response <code>audio/mpeg</code></summary>
                    <p>MP3 audio file containing the synthesized speech (if <code>Accept: audio/mpeg</code> header provided)</p>
                </details>
            </div>"#,
                sorted_langs.iter()
                    .map(|(tag, config)| {
                        let mut voices: Vec<_> = config.voices.iter().collect();
                        voices.sort_by_key(|(voice_id, _)| *voice_id);

                        let voices = voices
                            .iter()
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
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Start the web server
    Serve {
        /// Host to bind the server to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to run the server on
        #[arg(long, default_value_t = 4000)]
        port: u16,

        /// Path to the configuration file
        #[arg(long, env = "DIVVUN_CONFIG_PATH")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port, config } => {
            run_server(host, port, config).await?;
        }
    }

    Ok(())
}

async fn run_server(host: String, port: u16, config_path: String) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load config from file
    let config_contents = fs::read_to_string(&config_path)?;
    let languages: LanguagesConfig = toml::from_str(&config_contents)?;

    let app = Route::new()
        .at("/", get(index_get))
        .at("/health", get(health_get))
        .at("/languages", get(languages_get))
        .data(languages)
        .with(Cors::default());

    Server::new(TcpListener::bind((host, port)))
        .run(app)
        .await?;

    Ok(())
}
