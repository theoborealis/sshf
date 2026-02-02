use glob::Pattern;
use ssh_agent_lib::agent::Session;
use ssh_agent_lib::client::Client;
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::{Identity, SignRequest};
use ssh_key::public::KeyData;
use ssh_key::Signature;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixStream;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum FilterMode {
    Whitelist,
    Blacklist,
}

#[derive(Clone)]
pub struct FilterAgent {
    input_path: PathBuf,
    pattern: Pattern,
    mode: FilterMode,
    // Cache of allowed keys for sign verification (shared across clones in same session)
    allowed_keys: Arc<Mutex<Vec<KeyData>>>,
}

impl FilterAgent {
    pub fn new(input_path: PathBuf, pattern: String, mode: FilterMode) -> Self {
        let pattern = Pattern::new(&pattern).expect("Invalid glob pattern");
        Self {
            input_path,
            pattern,
            mode,
            allowed_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn matches(&self, comment: &str) -> bool {
        let matches_pattern = self.pattern.matches(comment);
        match self.mode {
            FilterMode::Whitelist => matches_pattern,
            FilterMode::Blacklist => !matches_pattern,
        }
    }

    async fn connect_upstream(&self) -> Result<Client<UnixStream>, AgentError> {
        let stream = UnixStream::connect(&self.input_path).await?;
        Ok(Client::new(stream))
    }
}

#[ssh_agent_lib::async_trait]
impl Session for FilterAgent {
    async fn request_identities(&mut self) -> Result<Vec<Identity>, AgentError> {
        let mut client = self.connect_upstream().await?;
        let identities = client.request_identities().await?;

        // Filter identities based on mode and pattern
        let filtered: Vec<Identity> = identities
            .into_iter()
            .filter(|id| self.matches(&id.comment))
            .collect();

        // Update allowed keys cache
        {
            let mut allowed = self.allowed_keys.lock().await;
            *allowed = filtered.iter().map(|id| id.pubkey.clone()).collect();
        }

        Ok(filtered)
    }

    async fn sign(&mut self, request: SignRequest) -> Result<Signature, AgentError> {
        // Check if this key is allowed
        let is_allowed = {
            let allowed = self.allowed_keys.lock().await;
            allowed.iter().any(|k| k == &request.pubkey)
        };

        if !is_allowed {
            // Key not in allowed list - refresh the list and check again
            let _ = self.request_identities().await?;

            let allowed = self.allowed_keys.lock().await;
            if !allowed.iter().any(|k| k == &request.pubkey) {
                return Err(AgentError::Failure);
            }
        }

        // Forward sign request to upstream
        let mut client = self.connect_upstream().await?;
        client.sign(request).await
    }
}
