use std::env;

mod error;

pub use error::{SendError, SetupError};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Mailer {
    from: String,
    messages_url: reqwest::Url,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageId(String);

impl Mailer {
    pub fn from_env() -> Result<Self, SetupError> {
        let domain = env::var("MAILER46_DOMAIN")
            .map_err(|_| SetupError::EnvVarMissing("MAILER46_DOMAIN"))?;
        let token =
            env::var("MAILER46_TOKEN").map_err(|_| SetupError::EnvVarMissing("MAILER46_TOKEN"))?;

        Self::new(domain, token)
    }

    /// Creates a new client operating against the given domain.
    /// Notice that the token must be the one provided by Mailgun, __NOT__ the
    /// base64 encoded api:<your token>.
    pub fn new(domain: impl AsRef<str>, token: impl AsRef<str>) -> Result<Self, SetupError> {
        let from = format!("noreply@{}", domain.as_ref());

        let messages_url = format!("https://api.eu.mailgun.net/v3/{}/messages", domain.as_ref())
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

    async fn send(&self, b: EmailBuilder) -> Result<MessageId, SendError> {
        if b.recipients.is_empty() {
            return Err(SendError::MissingField("to"));
        }

        let email = Email {
            from: self.from.clone(),
            to: b.recipients.join(","),
            subject: b.subject.unwrap_or_else(|| "no subject".into()),
            body: b.body,
        };

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
            println!("Body: {}", body);
            return Err(SendError::Non200Reply(status));
        }

        let reply = res.json::<MailReply>().await?;

        Ok(MessageId(reply.id))
    }
}

#[derive(serde::Deserialize)]
struct MailReply {
    id: String,
    message: String,
}

#[derive(serde::Deserialize)]
struct MailErrorReply {
    message: String,
}

#[derive(serde::Serialize)]
pub struct Email {
    from: String,
    to: String,
    subject: String,

    #[serde(flatten)]
    body: Option<Body>,
}

impl Email {
    pub fn build() -> EmailBuilder {
        EmailBuilder::default()
    }
}

#[derive(Debug, serde::Serialize)]
enum Body {
    #[serde(rename = "html")]
    Html(String),
    #[serde(rename = "text")]
    Text(String),
}

#[derive(Debug, Default)]
pub struct EmailBuilder {
    recipients: Vec<String>,
    subject: Option<String>,
    body: Option<Body>,
}

impl EmailBuilder {
    pub fn to(mut self, recipient: impl Into<String>) -> Self {
        self.recipients.push(recipient.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(Body::Text(body.into()));
        self
    }

    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.body = Some(Body::Html(html.into()));
        self
    }

    pub async fn send(self, client: &Mailer) -> Result<MessageId, SendError> {
        client.send(self).await
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serialize_email() {
        let email = Email {
            from: String::from("niclas"),
            to: String::from("someoneelse"),
            subject: String::from("Subject"),
            body: Some(Body::Html(String::from("HELLO"))),
        };

        let json = serde_json::to_string(&email).expect("Serializing email");

        assert_eq!(json, "asda");
    }

    #[tokio::test]
    async fn send_an_email() {
        let client = Mailer::from_env().expect("Creating client");

        let email = Email::build()
            .to("niclas@mobility46.se")
            .subject("test email!")
            .text_body("I'm a body used in a test somewhere")
            .send(&client)
            .await
            .expect("Building email");
    }
}
