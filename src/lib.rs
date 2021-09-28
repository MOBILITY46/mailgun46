//! ```
//!
//! use mailgun46::{Mailer, EmailBuilder};
//! // Setup a new client from env.
//! // The <from> header will be noreply@domain.
//! # async fn example() -> Result<(), Box<dyn std::error::Error + 'static>> {
//! let client = Mailer::from_env()?;
//! EmailBuilder::default()
//!   .to("somethingparseableasanemail")
//!   .subject("An email")
//!   .text_body("A plain, informative text body")
//!   .build()?
//!   .send(&client).await?;
//! # Ok(())
//! # }
//! ```
use std::env;

mod email;
mod error;

pub use {
    email::{Email, EmailBody, EmailBuilder},
    error::{BuildError, SendError, SetupError},
};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static MG_BASE_URL: &str = "https://api.eu.mailgun.net";

#[derive(Debug)]
pub struct Mailer {
    from: String,
    messages_url: reqwest::Url,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageId(String);

impl Mailer {
    /// Creates a new Mailer by reading from Environment variables:
    /// * `MAILER46_DOMAIN`: The domain to send from.
    /// * `MAILER46_TOKEN`: The raw token received from Mailgun.
    ///
    /// Uses base url to mailgun: `https://api.eu.mailgun.net`
    ///
    ///
    pub fn from_env() -> Result<Self, SetupError> {
        let domain = env::var("MAILER46_DOMAIN")
            .map_err(|_| SetupError::EnvVarMissing("MAILER46_DOMAIN"))?;
        let token =
            env::var("MAILER46_TOKEN").map_err(|_| SetupError::EnvVarMissing("MAILER46_TOKEN"))?;

        Self::new(domain, token)
    }

    /// Creates a new client operating against the given domain.
    /// Notice that the token must be the one provided by Mailgun, Mailer46 turns it into base64.
    ///
    /// Uses base url to mailgun: `https://api.eu.mailgun.net`
    pub fn new(domain: impl AsRef<str>, token: impl AsRef<str>) -> Result<Self, SetupError> {
        Self::new_with_mg_url(MG_BASE_URL, domain, token)
    }

    pub fn new_with_mg_url(
        mg_url: impl AsRef<str>,
        domain: impl AsRef<str>,
        token: impl AsRef<str>,
    ) -> Result<Self, SetupError> {
        let from = format!("noreply@{}", domain.as_ref());

        let messages_url = format!("{}/v3/{}/messages", mg_url.as_ref(), domain.as_ref())
            .parse::<reqwest::Url>()
            .map_err(|err| SetupError::InvalidVar("domain", err.to_string()))?;

        let mut headers = reqwest::header::HeaderMap::new();
        let token = base64::encode(format!("api:{}", token.as_ref()));
        let auth_value = reqwest::header::HeaderValue::from_str(&format!("Basic {}", token))
            .map_err(|err| SetupError::Build(err.to_string()))?;

        headers.insert("Authorization", auth_value);

        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .map_err(|err| SetupError::Build(err.to_string()))?;

        Ok(Self {
            from,
            messages_url,
            client,
        })
    }

    async fn send(&self, email: Email) -> Result<MessageId, SendError> {
        let res = self
            .client
            .post(self.messages_url.clone())
            .form(&email)
            .send()
            .await?;

        if res.status() != reqwest::StatusCode::OK {
            let status = res.status();
            let body_bs = res.bytes().await?;
            let body = String::from_utf8_lossy(&body_bs);
            return Err(SendError::Non200Reply {
                status,
                body: body.into(),
            });
        }

        let reply = res.json::<MailReply>().await?;

        Ok(MessageId(reply.id))
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct MailReply {
    id: String,
}

#[cfg(test)]
mod tests {

    use super::*;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    async fn setup() -> (Mailer, MockServer) {
        let server = MockServer::start().await;
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/v3/fakedomain/messages"))
            .and(matchers::header(
                "Authorization",
                "Basic YXBpOnRvbWF0b3Rva2Vu",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"
{
  "id": "<20210224131116.1.E5C867B3818DC87B@fakedomain>",
  "message": "Queued. Thank you."
}
"#,
            ))
            .mount(&server)
            .await;

        let client = Mailer::new_with_mg_url(&server.uri(), "fakedomain", "tomatotoken")
            .expect("Creating Mailer");
        (client, server)
    }

    #[test]
    fn serialize_email() {
        let email = Email {
            from: Some(String::from("niclas")),
            to: String::from("someoneelse"),
            subject: String::from("Subject"),
            body: Some(EmailBody::Html(String::from("HELLO"))),
        };

        let json = serde_json::to_string(&email).expect("Serializing email");

        assert_eq!(
            json,
            r#"{"from":"niclas","to":"someoneelse","subject":"Subject","html":"HELLO"}"#
        );
    }

    #[tokio::test]
    async fn send_a_test_email() {
        let (client, server) = setup().await;
        // let client = Mailer::from_env().expect("Creating client");

        let res = EmailBuilder::default()
            .to("niclas@mobility46.se")
            .subject("test email!")
            .text_body("I'm a body used in a test somewhere")
            .build()
            .expect("Building email")
            .send(&client)
            .await;

        assert!(
            res.is_ok(),
            "Error reply: {}\nServer got following requests:\n{}",
            res.err().unwrap(),
            server
                .received_requests()
                .await
                .map(|rqs| {
                    rqs.iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<String>>()
                        .join("\n\n")
                })
                .unwrap_or_else(|| String::from("-"))
        );
    }
}
