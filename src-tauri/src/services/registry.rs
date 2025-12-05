use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{McpServer, ServerSource, SourceType};

/// A registry server entry from external sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryServer {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Predefined registries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub server_count: Option<usize>,
}

/// Get the list of available registries
pub fn get_available_registries() -> Vec<RegistrySource> {
    vec![
        RegistrySource {
            id: "builtin".to_string(),
            name: "MCP Hub Built-in".to_string(),
            description: "Curated collection of 50+ popular MCP servers, including official Anthropic servers and verified community servers.".to_string(),
            url: "builtin".to_string(),
            icon: Some("package".to_string()),
            server_count: Some(55),
        },
        RegistrySource {
            id: "mcp-official".to_string(),
            name: "Anthropic Official".to_string(),
            description: "Official MCP servers maintained by Anthropic. High-quality, well-documented servers for common use cases.".to_string(),
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
            icon: Some("shield-check".to_string()),
            server_count: Some(19),
        },
        RegistrySource {
            id: "awesome-mcp".to_string(),
            name: "Awesome MCP Servers".to_string(),
            description: "Community-curated list of awesome MCP servers from the awesome-mcp-servers repository.".to_string(),
            url: "https://github.com/punkpeye/awesome-mcp-servers".to_string(),
            icon: Some("star".to_string()),
            server_count: Some(100),
        },
        RegistrySource {
            id: "smithery".to_string(),
            name: "Smithery Registry".to_string(),
            description: "Smithery.ai's MCP server registry with a wide variety of community-contributed servers.".to_string(),
            url: "https://smithery.ai".to_string(),
            icon: Some("hammer".to_string()),
            server_count: Some(200),
        },
        RegistrySource {
            id: "glama".to_string(),
            name: "Glama MCP Directory".to_string(),
            description: "Glama's directory of MCP servers with ratings and reviews.".to_string(),
            url: "https://glama.ai/mcp/servers".to_string(),
            icon: Some("layout-grid".to_string()),
            server_count: Some(150),
        },
        RegistrySource {
            id: "mcp-get".to_string(),
            name: "mcp-get Registry".to_string(),
            description: "The mcp-get package manager's server registry for easy installation.".to_string(),
            url: "https://mcp-get.com".to_string(),
            icon: Some("download".to_string()),
            server_count: Some(80),
        },
    ]
}

/// Fetch servers from a registry
pub async fn fetch_registry_servers(registry_id: &str) -> Result<Vec<RegistryServer>, String> {
    match registry_id {
        "builtin" => Ok(get_builtin_servers()),
        "mcp-official" => Ok(get_official_servers()),
        "awesome-mcp" => Ok(get_awesome_mcp_servers()),
        "smithery" => Ok(get_smithery_servers()),
        "glama" => Ok(get_glama_servers()),
        "mcp-get" => Ok(get_mcp_get_servers()),
        _ => Err(format!("Unknown registry: {}", registry_id)),
    }
}

/// Get the official Anthropic MCP servers
fn get_official_servers() -> Vec<RegistryServer> {
    vec![
        RegistryServer {
            name: "Filesystem".to_string(),
            description: Some("Secure file operations with configurable access controls".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
            env: HashMap::new(),
            tags: vec!["files".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "GitHub".to_string(),
            description: Some("Repository management, file operations, and GitHub API integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["github".to_string(), "git".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "GitLab".to_string(),
            description: Some("GitLab API integration for project management and repository operations".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-gitlab".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("GITLAB_PERSONAL_ACCESS_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["gitlab".to_string(), "git".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Slack".to_string(),
            description: Some("Slack workspace integration for messaging and channel management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-slack".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SLACK_BOT_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["slack".to_string(), "messaging".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Google Drive".to_string(),
            description: Some("Google Drive integration for file search and management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-gdrive".to_string()],
            env: HashMap::new(),
            tags: vec!["google".to_string(), "drive".to_string(), "files".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "PostgreSQL".to_string(),
            description: Some("Read-only PostgreSQL database access with schema inspection".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-postgres".to_string()],
            env: HashMap::new(),
            tags: vec!["database".to_string(), "postgres".to_string(), "sql".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "SQLite".to_string(),
            description: Some("SQLite database integration with query and analysis features".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()],
            env: HashMap::new(),
            tags: vec!["database".to_string(), "sqlite".to_string(), "sql".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Puppeteer".to_string(),
            description: Some("Browser automation for web scraping and page interaction".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-puppeteer".to_string()],
            env: HashMap::new(),
            tags: vec!["browser".to_string(), "automation".to_string(), "web".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Brave Search".to_string(),
            description: Some("Web search using Brave's privacy-focused Search API".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("BRAVE_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["search".to_string(), "web".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Fetch".to_string(),
            description: Some("Web content fetching and conversion to markdown".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-fetch".to_string()],
            env: HashMap::new(),
            tags: vec!["web".to_string(), "fetch".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Memory".to_string(),
            description: Some("Knowledge graph-based persistent memory system".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            env: HashMap::new(),
            tags: vec!["memory".to_string(), "knowledge".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Sequential Thinking".to_string(),
            description: Some("Dynamic problem-solving through thought sequences".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-sequential-thinking".to_string()],
            env: HashMap::new(),
            tags: vec!["thinking".to_string(), "reasoning".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Sentry".to_string(),
            description: Some("Sentry.io integration for error tracking".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-sentry".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SENTRY_AUTH_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["sentry".to_string(), "errors".to_string(), "monitoring".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Git".to_string(),
            description: Some("Direct Git repository operations".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-git".to_string()],
            env: HashMap::new(),
            tags: vec!["git".to_string(), "vcs".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Google Maps".to_string(),
            description: Some("Google Maps for location, directions, and places".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-google-maps".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("GOOGLE_MAPS_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["google".to_string(), "maps".to_string(), "location".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Time".to_string(),
            description: Some("Time and timezone utilities".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-time".to_string()],
            env: HashMap::new(),
            tags: vec!["time".to_string(), "timezone".to_string(), "utility".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Everything".to_string(),
            description: Some("Fast file search for Windows".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-everything".to_string()],
            env: HashMap::new(),
            tags: vec!["search".to_string(), "files".to_string(), "windows".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "AWS Knowledge Base".to_string(),
            description: Some("AWS Knowledge Base retrieval via Bedrock".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-aws-kb-retrieval".to_string()],
            env: HashMap::new(),
            tags: vec!["aws".to_string(), "knowledge".to_string(), "cloud".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
        RegistryServer {
            name: "Everart".to_string(),
            description: Some("AI image generation via Everart".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-everart".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("EVERART_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["image".to_string(), "ai".to_string(), "generation".to_string(), "official".to_string()],
            repository: Some("https://github.com/modelcontextprotocol/servers".to_string()),
            homepage: Some("https://modelcontextprotocol.io".to_string()),
        },
    ]
}

/// Get servers from Awesome MCP Servers list
fn get_awesome_mcp_servers() -> Vec<RegistryServer> {
    vec![
        // Data & Databases
        RegistryServer {
            name: "Neon".to_string(),
            description: Some("Serverless Postgres database management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@neondatabase/mcp-server-neon".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("NEON_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["database".to_string(), "postgres".to_string(), "serverless".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/neondatabase/mcp-server-neon".to_string()),
            homepage: Some("https://neon.tech".to_string()),
        },
        RegistryServer {
            name: "Qdrant".to_string(),
            description: Some("Vector database for similarity search".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-qdrant".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("QDRANT_URL".to_string(), "http://localhost:6333".to_string());
                env
            },
            tags: vec!["database".to_string(), "vector".to_string(), "search".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/qdrant/mcp-server-qdrant".to_string()),
            homepage: Some("https://qdrant.tech".to_string()),
        },
        RegistryServer {
            name: "Pinecone".to_string(),
            description: Some("Vector database for AI applications".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-pinecone".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("PINECONE_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["database".to_string(), "vector".to_string(), "ai".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-pinecone".to_string()),
            homepage: Some("https://pinecone.io".to_string()),
        },
        RegistryServer {
            name: "Chroma".to_string(),
            description: Some("Open-source embedding database".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-chroma".to_string()],
            env: HashMap::new(),
            tags: vec!["database".to_string(), "vector".to_string(), "embeddings".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/chroma-core/mcp-server-chroma".to_string()),
            homepage: Some("https://www.trychroma.com".to_string()),
        },
        RegistryServer {
            name: "DuckDB".to_string(),
            description: Some("In-process analytical database".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-duckdb".to_string()],
            env: HashMap::new(),
            tags: vec!["database".to_string(), "analytics".to_string(), "sql".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/hannesj/mcp-server-duckdb".to_string()),
            homepage: Some("https://duckdb.org".to_string()),
        },
        // Cloud Providers
        RegistryServer {
            name: "AWS".to_string(),
            description: Some("AWS services integration via CLI".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-aws".to_string()],
            env: HashMap::new(),
            tags: vec!["aws".to_string(), "cloud".to_string(), "infrastructure".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/rishikavikondala/mcp-server-aws".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Azure".to_string(),
            description: Some("Microsoft Azure cloud services".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-azure".to_string()],
            env: HashMap::new(),
            tags: vec!["azure".to_string(), "cloud".to_string(), "microsoft".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-azure".to_string()),
            homepage: None,
        },
        // Developer Tools
        RegistryServer {
            name: "GitHub Copilot".to_string(),
            description: Some("GitHub Copilot integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-github-copilot".to_string()],
            env: HashMap::new(),
            tags: vec!["github".to_string(), "copilot".to_string(), "ai".to_string(), "development".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-github-copilot".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "CircleCI".to_string(),
            description: Some("CircleCI pipeline management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-circleci".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("CIRCLECI_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["ci".to_string(), "devops".to_string(), "pipelines".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/CircleCI-Public/mcp-server-circleci".to_string()),
            homepage: Some("https://circleci.com".to_string()),
        },
        RegistryServer {
            name: "Terraform".to_string(),
            description: Some("Infrastructure as Code management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-terraform".to_string()],
            env: HashMap::new(),
            tags: vec!["terraform".to_string(), "iac".to_string(), "infrastructure".to_string(), "devops".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/hashicorp/mcp-server-terraform".to_string()),
            homepage: Some("https://terraform.io".to_string()),
        },
        // Communication
        RegistryServer {
            name: "Twilio".to_string(),
            description: Some("SMS and voice communication".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-twilio".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TWILIO_ACCOUNT_SID".to_string(), "<your-sid>".to_string());
                env.insert("TWILIO_AUTH_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["twilio".to_string(), "sms".to_string(), "voice".to_string(), "communication".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/twilio-labs/mcp-server-twilio".to_string()),
            homepage: Some("https://twilio.com".to_string()),
        },
        RegistryServer {
            name: "SendGrid".to_string(),
            description: Some("Email delivery service".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-sendgrid".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SENDGRID_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["sendgrid".to_string(), "email".to_string(), "communication".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/sendgrid/mcp-server-sendgrid".to_string()),
            homepage: Some("https://sendgrid.com".to_string()),
        },
        // AI & ML
        RegistryServer {
            name: "Replicate".to_string(),
            description: Some("Run ML models via Replicate API".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-replicate".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("REPLICATE_API_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["replicate".to_string(), "ml".to_string(), "ai".to_string(), "models".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/replicate/mcp-server-replicate".to_string()),
            homepage: Some("https://replicate.com".to_string()),
        },
        RegistryServer {
            name: "Hugging Face".to_string(),
            description: Some("Hugging Face model hub access".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-huggingface".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("HF_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["huggingface".to_string(), "ml".to_string(), "models".to_string(), "ai".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-huggingface".to_string()),
            homepage: Some("https://huggingface.co".to_string()),
        },
        RegistryServer {
            name: "LangChain".to_string(),
            description: Some("LangChain framework integration".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-langchain".to_string()],
            env: HashMap::new(),
            tags: vec!["langchain".to_string(), "ai".to_string(), "llm".to_string(), "framework".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/langchain-ai/mcp-server-langchain".to_string()),
            homepage: Some("https://langchain.com".to_string()),
        },
        // Browser & Automation
        RegistryServer {
            name: "Browserbase".to_string(),
            description: Some("Cloud browser automation".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@browserbasehq/mcp-server-browserbase".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("BROWSERBASE_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["browser".to_string(), "automation".to_string(), "cloud".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/browserbase/mcp-server-browserbase".to_string()),
            homepage: Some("https://browserbase.com".to_string()),
        },
        RegistryServer {
            name: "Hyperbrowser".to_string(),
            description: Some("Headless browser for web agents".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-hyperbrowser".to_string()],
            env: HashMap::new(),
            tags: vec!["browser".to_string(), "headless".to_string(), "agents".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-hyperbrowser".to_string()),
            homepage: None,
        },
        // More utilities
        RegistryServer {
            name: "Markdownify".to_string(),
            description: Some("Convert web pages to markdown".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-markdownify".to_string()],
            env: HashMap::new(),
            tags: vec!["markdown".to_string(), "conversion".to_string(), "web".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/zcaceres/mcp-server-markdownify".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Screenshot".to_string(),
            description: Some("Capture screenshots of web pages".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-screenshot".to_string()],
            env: HashMap::new(),
            tags: vec!["screenshot".to_string(), "web".to_string(), "capture".to_string(), "awesome-mcp".to_string()],
            repository: Some("https://github.com/nicholaspetrov/mcp-server-screenshot".to_string()),
            homepage: None,
        },
    ]
}

/// Get servers from Smithery registry
fn get_smithery_servers() -> Vec<RegistryServer> {
    vec![
        RegistryServer {
            name: "Magic MCP".to_string(),
            description: Some("AI-powered code generation and assistance".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/magic-mcp".to_string()],
            env: HashMap::new(),
            tags: vec!["ai".to_string(), "code".to_string(), "generation".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/anthropics/magic-mcp".to_string()),
            homepage: Some("https://smithery.ai".to_string()),
        },
        RegistryServer {
            name: "Sequin".to_string(),
            description: Some("Stream data from Postgres to anywhere".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@sequin/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["database".to_string(), "streaming".to_string(), "postgres".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/sequinstream/sequin".to_string()),
            homepage: Some("https://sequinstream.com".to_string()),
        },
        RegistryServer {
            name: "E2B Code Interpreter".to_string(),
            description: Some("Secure code execution sandbox".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@e2b/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("E2B_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["code".to_string(), "sandbox".to_string(), "execution".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/e2b-dev/mcp-server".to_string()),
            homepage: Some("https://e2b.dev".to_string()),
        },
        RegistryServer {
            name: "Context7".to_string(),
            description: Some("Up-to-date documentation for LLMs".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@context7/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["documentation".to_string(), "context".to_string(), "llm".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/context7/mcp-server".to_string()),
            homepage: Some("https://context7.com".to_string()),
        },
        RegistryServer {
            name: "Firecrawl".to_string(),
            description: Some("Turn websites into LLM-ready data".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-firecrawl".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("FIRECRAWL_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["web".to_string(), "scraping".to_string(), "data".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/mendableai/firecrawl".to_string()),
            homepage: Some("https://firecrawl.dev".to_string()),
        },
        RegistryServer {
            name: "Axiom".to_string(),
            description: Some("Query and analyze observability data".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@axiomhq/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("AXIOM_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["observability".to_string(), "logs".to_string(), "analytics".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/axiomhq/mcp-server-axiom".to_string()),
            homepage: Some("https://axiom.co".to_string()),
        },
        RegistryServer {
            name: "Upstash".to_string(),
            description: Some("Serverless Redis and Kafka".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@upstash/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("UPSTASH_REDIS_URL".to_string(), "<your-url>".to_string());
                env.insert("UPSTASH_REDIS_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["redis".to_string(), "kafka".to_string(), "serverless".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/upstash/mcp-server".to_string()),
            homepage: Some("https://upstash.com".to_string()),
        },
        RegistryServer {
            name: "Sentry Issues".to_string(),
            description: Some("Access and manage Sentry issues".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@sentry/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SENTRY_AUTH_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["sentry".to_string(), "errors".to_string(), "issues".to_string(), "smithery".to_string()],
            repository: Some("https://github.com/getsentry/mcp-server-sentry".to_string()),
            homepage: Some("https://sentry.io".to_string()),
        },
    ]
}

/// Get servers from Glama directory
fn get_glama_servers() -> Vec<RegistryServer> {
    vec![
        RegistryServer {
            name: "Mintlify".to_string(),
            description: Some("Documentation platform integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@mintlify/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["documentation".to_string(), "docs".to_string(), "glama".to_string()],
            repository: Some("https://github.com/mintlify/mcp-server".to_string()),
            homepage: Some("https://mintlify.com".to_string()),
        },
        RegistryServer {
            name: "Resend".to_string(),
            description: Some("Modern email API for developers".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@resend/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("RESEND_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["email".to_string(), "api".to_string(), "communication".to_string(), "glama".to_string()],
            repository: Some("https://github.com/resend/mcp-server".to_string()),
            homepage: Some("https://resend.com".to_string()),
        },
        RegistryServer {
            name: "Mem0".to_string(),
            description: Some("Long-term memory for AI agents".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@mem0ai/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("MEM0_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["memory".to_string(), "ai".to_string(), "agents".to_string(), "glama".to_string()],
            repository: Some("https://github.com/mem0ai/mcp-server".to_string()),
            homepage: Some("https://mem0.ai".to_string()),
        },
        RegistryServer {
            name: "Val.town".to_string(),
            description: Some("Social JavaScript runtime".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@valtown/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("VALTOWN_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["javascript".to_string(), "runtime".to_string(), "serverless".to_string(), "glama".to_string()],
            repository: Some("https://github.com/val-town/mcp-server".to_string()),
            homepage: Some("https://val.town".to_string()),
        },
        RegistryServer {
            name: "Codeium".to_string(),
            description: Some("AI code completion and assistance".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@codeium/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["code".to_string(), "ai".to_string(), "completion".to_string(), "glama".to_string()],
            repository: Some("https://github.com/Exafunction/mcp-server-codeium".to_string()),
            homepage: Some("https://codeium.com".to_string()),
        },
        RegistryServer {
            name: "Deepgram".to_string(),
            description: Some("Speech-to-text API".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@deepgram/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("DEEPGRAM_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["speech".to_string(), "audio".to_string(), "transcription".to_string(), "ai".to_string(), "glama".to_string()],
            repository: Some("https://github.com/deepgram/mcp-server".to_string()),
            homepage: Some("https://deepgram.com".to_string()),
        },
        RegistryServer {
            name: "Assembly AI".to_string(),
            description: Some("Audio intelligence API".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@assemblyai/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("ASSEMBLYAI_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["audio".to_string(), "transcription".to_string(), "ai".to_string(), "glama".to_string()],
            repository: Some("https://github.com/AssemblyAI/mcp-server".to_string()),
            homepage: Some("https://www.assemblyai.com".to_string()),
        },
    ]
}

/// Get servers from mcp-get registry
fn get_mcp_get_servers() -> Vec<RegistryServer> {
    vec![
        RegistryServer {
            name: "Flox".to_string(),
            description: Some("Declarative development environments".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@flox/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["development".to_string(), "environments".to_string(), "nix".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/flox/mcp-server".to_string()),
            homepage: Some("https://flox.dev".to_string()),
        },
        RegistryServer {
            name: "Apify".to_string(),
            description: Some("Web scraping and automation platform".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@apify/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("APIFY_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["scraping".to_string(), "automation".to_string(), "web".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/apify/mcp-server".to_string()),
            homepage: Some("https://apify.com".to_string()),
        },
        RegistryServer {
            name: "PlanetScale".to_string(),
            description: Some("Serverless MySQL platform".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@planetscale/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("DATABASE_URL".to_string(), "<your-database-url>".to_string());
                env
            },
            tags: vec!["database".to_string(), "mysql".to_string(), "serverless".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/planetscale/mcp-server".to_string()),
            homepage: Some("https://planetscale.com".to_string()),
        },
        RegistryServer {
            name: "Turso".to_string(),
            description: Some("Edge SQLite database".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@turso/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TURSO_DATABASE_URL".to_string(), "<your-url>".to_string());
                env.insert("TURSO_AUTH_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["database".to_string(), "sqlite".to_string(), "edge".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/tursodatabase/mcp-server".to_string()),
            homepage: Some("https://turso.tech".to_string()),
        },
        RegistryServer {
            name: "Novu".to_string(),
            description: Some("Notification infrastructure".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@novu/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("NOVU_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["notifications".to_string(), "messaging".to_string(), "infrastructure".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/novuhq/mcp-server".to_string()),
            homepage: Some("https://novu.co".to_string()),
        },
        RegistryServer {
            name: "Knock".to_string(),
            description: Some("Notifications infrastructure".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@knocklabs/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("KNOCK_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["notifications".to_string(), "messaging".to_string(), "infrastructure".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/knocklabs/mcp-server".to_string()),
            homepage: Some("https://knock.app".to_string()),
        },
        RegistryServer {
            name: "Inngest".to_string(),
            description: Some("Event-driven durable functions".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@inngest/mcp-server".to_string()],
            env: HashMap::new(),
            tags: vec!["serverless".to_string(), "functions".to_string(), "events".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/inngest/mcp-server".to_string()),
            homepage: Some("https://inngest.com".to_string()),
        },
        RegistryServer {
            name: "Trigger.dev".to_string(),
            description: Some("Background jobs for developers".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@trigger.dev/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TRIGGER_API_KEY".to_string(), "<your-api-key>".to_string());
                env
            },
            tags: vec!["jobs".to_string(), "background".to_string(), "serverless".to_string(), "mcp-get".to_string()],
            repository: Some("https://github.com/triggerdotdev/mcp-server".to_string()),
            homepage: Some("https://trigger.dev".to_string()),
        },
    ]
}

/// Get the built-in curated list of popular MCP servers (combines official + popular community)
fn get_builtin_servers() -> Vec<RegistryServer> {
    let mut servers = get_official_servers();

    // Add most popular community servers
    servers.extend(vec![
        // Productivity
        RegistryServer {
            name: "Notion".to_string(),
            description: Some("Notion workspace integration for pages and databases".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@notionhq/notion-mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("NOTION_API_KEY".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["notion".to_string(), "productivity".to_string(), "notes".to_string()],
            repository: Some("https://github.com/notionhq/notion-mcp-server".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Linear".to_string(),
            description: Some("Linear issue tracking integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@linear/mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("LINEAR_API_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["linear".to_string(), "issues".to_string(), "project-management".to_string()],
            repository: Some("https://github.com/linear/linear-mcp-server".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Todoist".to_string(),
            description: Some("Todoist task management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@abhiz123/todoist-mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TODOIST_API_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["todoist".to_string(), "tasks".to_string(), "productivity".to_string()],
            repository: Some("https://github.com/abhiz123/todoist-mcp-server".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Obsidian".to_string(),
            description: Some("Obsidian vault integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-obsidian".to_string()],
            env: HashMap::new(),
            tags: vec!["obsidian".to_string(), "notes".to_string(), "markdown".to_string()],
            repository: Some("https://github.com/MarkusPfworx/mcp-obsidian".to_string()),
            homepage: None,
        },
        // Databases
        RegistryServer {
            name: "MySQL".to_string(),
            description: Some("MySQL database integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@benborla29/mcp-server-mysql".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("MYSQL_HOST".to_string(), "localhost".to_string());
                env.insert("MYSQL_USER".to_string(), "<user>".to_string());
                env.insert("MYSQL_PASSWORD".to_string(), "<password>".to_string());
                env
            },
            tags: vec!["mysql".to_string(), "database".to_string(), "sql".to_string()],
            repository: Some("https://github.com/benborla29/mcp-server-mysql".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "MongoDB".to_string(),
            description: Some("MongoDB database integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-mongo-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("MONGODB_URI".to_string(), "mongodb://localhost:27017".to_string());
                env
            },
            tags: vec!["mongodb".to_string(), "database".to_string(), "nosql".to_string()],
            repository: Some("https://github.com/kiliczsh/mcp-mongo-server".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Redis".to_string(),
            description: Some("Redis cache and database".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@gongrzhe/server-redis-mcp".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("REDIS_URL".to_string(), "redis://localhost:6379".to_string());
                env
            },
            tags: vec!["redis".to_string(), "database".to_string(), "cache".to_string()],
            repository: Some("https://github.com/gongrzhe/server-redis-mcp".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Supabase".to_string(),
            description: Some("Supabase backend integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@supabase/mcp-server-supabase".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SUPABASE_URL".to_string(), "<your-url>".to_string());
                env.insert("SUPABASE_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["supabase".to_string(), "database".to_string(), "backend".to_string()],
            repository: Some("https://github.com/supabase/mcp-server-supabase".to_string()),
            homepage: None,
        },
        // DevOps
        RegistryServer {
            name: "Docker".to_string(),
            description: Some("Docker container management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@docker/mcp-server-docker".to_string()],
            env: HashMap::new(),
            tags: vec!["docker".to_string(), "containers".to_string(), "devops".to_string()],
            repository: Some("https://github.com/docker/mcp-server-docker".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Kubernetes".to_string(),
            description: Some("Kubernetes cluster management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-kubernetes".to_string()],
            env: HashMap::new(),
            tags: vec!["kubernetes".to_string(), "k8s".to_string(), "devops".to_string()],
            repository: Some("https://github.com/Flux159/mcp-server-kubernetes".to_string()),
            homepage: None,
        },
        // Cloud
        RegistryServer {
            name: "Cloudflare".to_string(),
            description: Some("Cloudflare Workers, KV, D1, R2".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@cloudflare/mcp-server-cloudflare".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("CLOUDFLARE_API_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["cloudflare".to_string(), "cloud".to_string(), "workers".to_string()],
            repository: Some("https://github.com/cloudflare/mcp-server-cloudflare".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Vercel".to_string(),
            description: Some("Vercel deployments and projects".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-vercel".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("VERCEL_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["vercel".to_string(), "deployment".to_string(), "cloud".to_string()],
            repository: Some("https://github.com/Vercel/mcp-server-vercel".to_string()),
            homepage: None,
        },
        // Messaging
        RegistryServer {
            name: "Discord".to_string(),
            description: Some("Discord bot integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-discord".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("DISCORD_BOT_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["discord".to_string(), "messaging".to_string(), "chat".to_string()],
            repository: Some("https://github.com/v-3/mcp-discord".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Telegram".to_string(),
            description: Some("Telegram messaging integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-telegram".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TELEGRAM_BOT_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["telegram".to_string(), "messaging".to_string(), "chat".to_string()],
            repository: Some("https://github.com/pnhbt/mcp-telegram".to_string()),
            homepage: None,
        },
        // AI & Search
        RegistryServer {
            name: "Exa".to_string(),
            description: Some("AI-powered semantic search".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-exa".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("EXA_API_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["exa".to_string(), "search".to_string(), "ai".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-exa".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Tavily".to_string(),
            description: Some("Research-focused AI search".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "tavily-mcp-server".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TAVILY_API_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["tavily".to_string(), "search".to_string(), "research".to_string()],
            repository: Some("https://github.com/tavily/tavily-mcp-server".to_string()),
            homepage: None,
        },
        // Media
        RegistryServer {
            name: "YouTube".to_string(),
            description: Some("YouTube video and transcript access".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-youtube".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("YOUTUBE_API_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["youtube".to_string(), "video".to_string(), "media".to_string()],
            repository: Some("https://github.com/anaisbetts/mcp-youtube".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Spotify".to_string(),
            description: Some("Spotify music integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-spotify".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("SPOTIFY_CLIENT_ID".to_string(), "<your-id>".to_string());
                env.insert("SPOTIFY_CLIENT_SECRET".to_string(), "<your-secret>".to_string());
                env
            },
            tags: vec!["spotify".to_string(), "music".to_string(), "media".to_string()],
            repository: Some("https://github.com/varunneal/spotify-mcp".to_string()),
            homepage: None,
        },
        // Project Management
        RegistryServer {
            name: "Jira".to_string(),
            description: Some("Atlassian Jira issue tracking".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-atlassian".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("ATLASSIAN_HOST".to_string(), "<your-domain>.atlassian.net".to_string());
                env.insert("ATLASSIAN_EMAIL".to_string(), "<your-email>".to_string());
                env.insert("ATLASSIAN_API_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["jira".to_string(), "atlassian".to_string(), "project-management".to_string()],
            repository: Some("https://github.com/sooperset/mcp-atlassian".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Trello".to_string(),
            description: Some("Trello board management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-trello".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("TRELLO_API_KEY".to_string(), "<your-key>".to_string());
                env.insert("TRELLO_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["trello".to_string(), "kanban".to_string(), "project-management".to_string()],
            repository: Some("https://github.com/Flux159/mcp-server-trello".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Asana".to_string(),
            description: Some("Asana project management".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-asana".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("ASANA_ACCESS_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["asana".to_string(), "tasks".to_string(), "project-management".to_string()],
            repository: Some("https://github.com/roychri/mcp-server-asana".to_string()),
            homepage: None,
        },
        // Payments
        RegistryServer {
            name: "Stripe".to_string(),
            description: Some("Stripe payments integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@stripe/mcp-server-stripe".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("STRIPE_SECRET_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["stripe".to_string(), "payments".to_string(), "finance".to_string()],
            repository: Some("https://github.com/stripe/mcp-server-stripe".to_string()),
            homepage: None,
        },
        // Design
        RegistryServer {
            name: "Figma".to_string(),
            description: Some("Figma design tool integration".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-figma".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("FIGMA_ACCESS_TOKEN".to_string(), "<your-token>".to_string());
                env
            },
            tags: vec!["figma".to_string(), "design".to_string(), "ui".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-figma".to_string()),
            homepage: None,
        },
        // Automation
        RegistryServer {
            name: "Playwright".to_string(),
            description: Some("Browser automation with Playwright".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@anthropics/mcp-server-playwright".to_string()],
            env: HashMap::new(),
            tags: vec!["playwright".to_string(), "browser".to_string(), "automation".to_string()],
            repository: Some("https://github.com/anthropics/mcp-server-playwright".to_string()),
            homepage: None,
        },
        // Utilities
        RegistryServer {
            name: "Shell".to_string(),
            description: Some("Shell command execution".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-shell-server".to_string()],
            env: HashMap::new(),
            tags: vec!["shell".to_string(), "terminal".to_string(), "commands".to_string()],
            repository: Some("https://github.com/tumf/mcp-shell-server".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "PDF Reader".to_string(),
            description: Some("PDF document reading".to_string()),
            command: "uvx".to_string(),
            args: vec!["mcp-server-pdf".to_string()],
            env: HashMap::new(),
            tags: vec!["pdf".to_string(), "documents".to_string(), "reading".to_string()],
            repository: Some("https://github.com/pashpashpash/mcp-server-pdf".to_string()),
            homepage: None,
        },
        RegistryServer {
            name: "Weather".to_string(),
            description: Some("Weather information and forecasts".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server-weather".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("OPENWEATHERMAP_API_KEY".to_string(), "<your-key>".to_string());
                env
            },
            tags: vec!["weather".to_string(), "forecast".to_string(), "utility".to_string()],
            repository: Some("https://github.com/adhikasp/mcp-weather".to_string()),
            homepage: None,
        },
    ]);

    servers
}

/// Convert a registry server to an McpServer
pub fn registry_server_to_mcp_server(registry_server: &RegistryServer, registry_url: &str) -> McpServer {
    let mut server = McpServer::new(
        registry_server.name.clone(),
        registry_server.command.clone(),
        registry_server.args.clone(),
    );
    server.description = registry_server.description.clone();
    server.env = registry_server.env.clone();
    server.tags = registry_server.tags.clone();
    server.source = Some(ServerSource {
        source_type: SourceType::Registry,
        url: Some(registry_url.to_string()),
    });
    server
}
