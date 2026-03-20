use serde::{Deserialize, Serialize};

/// Bolagsverket API base URLs.
const PROD_BASE: &str = "https://api.bolagsverket.se/lamna-in-arsredovisning/v2.1";
const TEST_BASE: &str = "https://api-test.bolagsverket.se/lamna-in-arsredovisning/v2.1";

/// Bolagsverket filing API client.
pub struct BolagsverketClient {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Token for verification step
    pub kontrolltoken: Option<String>,
    /// Token for submission step
    pub inlamningtoken: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub status: String,
    /// Errors found during verification
    #[serde(default)]
    pub fel: Vec<FilingIssue>,
    /// Warnings found during verification
    #[serde(default)]
    pub varningar: Vec<FilingIssue>,
    /// Submission token (available if verification passed)
    pub inlamningtoken: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilingIssue {
    pub kod: Option<String>,
    pub meddelande: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResponse {
    pub status: String,
    /// Filing reference number from Bolagsverket
    pub arendenummer: Option<String>,
    #[serde(default)]
    pub fel: Vec<FilingIssue>,
}

/// Result of the full filing flow.
#[derive(Debug, Clone, Serialize)]
pub struct FilingResult {
    pub step: FilingStep,
    pub success: bool,
    pub message: String,
    pub verification_errors: Vec<FilingIssue>,
    pub verification_warnings: Vec<FilingIssue>,
    pub submission_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStep {
    TokenCreated,
    Verified,
    Submitted,
    Failed,
}

impl BolagsverketClient {
    pub fn new(use_test: bool) -> Self {
        let base_url = if use_test {
            TEST_BASE.to_string()
        } else {
            PROD_BASE.to_string()
        };

        Self {
            base_url,
            http: reqwest::Client::new(),
        }
    }

    /// Step 1: Create a submission token.
    pub async fn create_token(&self, ixbrl_content: &str) -> Result<TokenResponse, FilingError> {
        let url = format!("{}/skapa-inlamningtoken", self.base_url);

        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/xhtml+xml; charset=utf-8")
            .body(ixbrl_content.to_string())
            .send()
            .await
            .map_err(|e| FilingError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FilingError::Api(format!(
                "Token creation failed ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| FilingError::Parse(e.to_string()))
    }

    /// Step 2: Verify the annual report.
    pub async fn verify(
        &self,
        token: &str,
        ixbrl_content: &str,
    ) -> Result<VerificationResponse, FilingError> {
        let url = format!("{}/kontrollera/{}", self.base_url, token);

        let response = self
            .http
            .put(&url)
            .header("Content-Type", "application/xhtml+xml; charset=utf-8")
            .body(ixbrl_content.to_string())
            .send()
            .await
            .map_err(|e| FilingError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FilingError::Api(format!(
                "Verification failed ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| FilingError::Parse(e.to_string()))
    }

    /// Step 3: Submit the annual report.
    pub async fn submit(&self, token: &str) -> Result<SubmissionResponse, FilingError> {
        let url = format!("{}/inlamning/{}", self.base_url, token);

        let response = self
            .http
            .post(&url)
            .send()
            .await
            .map_err(|e| FilingError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FilingError::Api(format!(
                "Submission failed ({status}): {body}"
            )));
        }

        response
            .json()
            .await
            .map_err(|e| FilingError::Parse(e.to_string()))
    }

    /// Execute the full filing flow: token → verify → submit.
    pub async fn file_annual_report(
        &self,
        ixbrl_content: &str,
    ) -> Result<FilingResult, FilingError> {
        // Step 1: Create token
        let token_resp = self.create_token(ixbrl_content).await?;
        let kontrolltoken = token_resp.kontrolltoken.ok_or_else(|| {
            FilingError::Api("No kontrolltoken in response".to_string())
        })?;

        // Step 2: Verify
        let verify_resp = self.verify(&kontrolltoken, ixbrl_content).await?;

        if !verify_resp.fel.is_empty() {
            return Ok(FilingResult {
                step: FilingStep::Failed,
                success: false,
                message: format!(
                    "Verifieringen misslyckades med {} fel",
                    verify_resp.fel.len()
                ),
                verification_errors: verify_resp.fel,
                verification_warnings: verify_resp.varningar,
                submission_reference: None,
            });
        }

        let inlamningtoken = verify_resp.inlamningtoken.ok_or_else(|| {
            FilingError::Api("No inlamningtoken after verification".to_string())
        })?;

        // Step 3: Submit
        let submit_resp = self.submit(&inlamningtoken).await?;

        if !submit_resp.fel.is_empty() {
            return Ok(FilingResult {
                step: FilingStep::Failed,
                success: false,
                message: format!(
                    "Inlämningen misslyckades med {} fel",
                    submit_resp.fel.len()
                ),
                verification_errors: submit_resp.fel,
                verification_warnings: Vec::new(),
                submission_reference: None,
            });
        }

        Ok(FilingResult {
            step: FilingStep::Submitted,
            success: true,
            message: format!(
                "Årsredovisningen har lämnats in. Ärendenummer: {}",
                submit_resp
                    .arendenummer
                    .as_deref()
                    .unwrap_or("(inget nummer)")
            ),
            verification_errors: Vec::new(),
            verification_warnings: verify_resp.varningar,
            submission_reference: submit_resp.arendenummer,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FilingError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<FilingError> for crate::error::AppError {
    fn from(e: FilingError) -> Self {
        crate::error::AppError::Internal(e.to_string())
    }
}
