use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    tts: ConfigTts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigTts {
    port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceConfig {
    name: String,
    port: u16,
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
    model: String,
    #[serde(default)]
    speaker: Option<u32>,
    #[serde(default)]
    language: Option<u32>,
}

#[handler]
async fn languages_get(Data(languages): Data<&LanguagesConfig>) -> impl IntoResponse {
    // TODO: remove the config layer
    Json(serde_json::json!({ "available": languages })).into_response()
}

#[handler]
async fn health_get() -> impl IntoResponse {
    Json(json!({ "status": "ok" })).into_response()
}

#[handler]
async fn index_get(Data(languages): Data<&LanguagesConfig>) -> impl IntoResponse {
    let mut html = include_str!("../index.html").to_string();

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
    },
    /// Generate nginx configuration files
    Generate {
        /// Directory path to output the configuration files
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port } => {
            run_server(host, port).await?;
        }
        Commands::Generate { path } => {
            // Parse languages from TOML
            let languages: LanguagesConfig = toml::from_str(LANGUAGES)?;

            // Create directory if it doesn't exist
            fs::create_dir_all(&path)?;

            // Write nginx locations config
            let nginx_config = generate_nginx_config(&languages);
            let nginx_path = Path::new(&path).join("locations.conf");
            fs::write(nginx_path, nginx_config)?;

            // Write proxy headers config
            let proxy_headers = generate_proxy_headers_config();
            let proxy_path = Path::new(&path).join("proxy-headers.conf");
            fs::write(proxy_path, proxy_headers)?;

            println!("Generated configuration files in: {}", path);
        }
    }

    Ok(())
}

const LANGUAGES: &str = include_str!("../languages.toml");

async fn run_server(host: String, port: u16) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Parse languages from TOML
    let languages: LanguagesConfig = toml::from_str(LANGUAGES)?;

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

fn generate_nginx_config(languages: &LanguagesConfig) -> String {
    let mut configs = Vec::new();

    // Generate grammar service configs
    let mut grammar_services: Vec<_> = languages.grammar.iter().collect();
    grammar_services.sort_by_key(|(tag, _)| *tag);
    for (tag, service) in grammar_services {
        configs.push(generate_location_block(
            &format!("/grammar/{}", tag),
            service.port,
            "",
            &HashMap::new(),
        ));
    }

    // Generate speller service configs
    let mut speller_services: Vec<_> = languages.speller.iter().collect();
    speller_services.sort_by_key(|(tag, _)| *tag);
    for (tag, service) in speller_services {
        configs.push(generate_location_block(
            &format!("/speller/{}", tag),
            service.port,
            "",
            &HashMap::new(),
        ));
    }

    // Generate hyphenation service configs
    let mut hyphenation_services: Vec<_> = languages.hyphenation.iter().collect();
    hyphenation_services.sort_by_key(|(tag, _)| *tag);
    for (tag, service) in hyphenation_services {
        configs.push(generate_location_block(
            &format!("/hyphenation/{}", tag),
            service.port,
            "",
            &HashMap::new(),
        ));
    }

    // Generate TTS service configs
    let mut tts_services: Vec<_> = languages.tts.iter().collect();
    tts_services.sort_by_key(|(tag, _)| *tag);
    for (tag, tts_config) in tts_services {
        let mut voices: Vec<_> = tts_config.voices.iter().collect();
        voices.sort_by_key(|(voice_id, _)| *voice_id);
        for (voice_id, voice) in voices {
            let mut query = HashMap::new();
            if let Some(language) = voice.language {
                query.insert("language".to_string(), language.to_string());
            }
            if let Some(speaker) = voice.speaker {
                query.insert("speaker".to_string(), speaker.to_string());
            }
            configs.push(generate_location_block(
                &format!("/tts/{}/{}", tag, voice_id),
                languages.config.tts.port,
                &voice.model,
                &query,
            ));
        }
    }

    configs.join("\n\n")
}

fn generate_location_block(
    fe_path: &str,
    port: u16,
    be_path: &str,
    query: &HashMap<String, String>,
) -> String {
    let mut query = query
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    if !query.is_empty() {
        query = format!("?{}", query);
    }

    format!(
        r#"location {} {{
    proxy_pass http://127.0.0.1:{}/{}{};
    include proxy-headers.conf;
}}"#,
        fe_path, port, be_path, query
    )
}

fn generate_proxy_headers_config() -> String {
    r#"proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection 'upgrade';
proxy_set_header Host $host;
proxy_cache_bypass $http_upgrade;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;"#
        .to_string()
}
