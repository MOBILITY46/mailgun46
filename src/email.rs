use crate::{BuildError, Mailer, MessageId, SendError};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Email {
    /// Optional, only used if set. If None the from is taken from Mailer.
    pub(crate) from: Option<String>,
    pub(crate) to: String,
    pub(crate) subject: String,

    #[serde(flatten)]
    pub(crate) body: Option<EmailBody>,
}

impl Email {
    pub async fn send(self, mailer: &Mailer) -> Result<MessageId, SendError> {
        mailer.send(self).await
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum EmailBody {
    #[serde(rename = "html")]
    Html(String),
    #[serde(rename = "text")]
    Text(String),
}

#[derive(Debug, Default)]
pub struct EmailBuilder {
    from: Option<String>,
    recipients: Vec<String>,
    subject: Option<String>,
    body: Option<EmailBody>,
}

impl EmailBuilder {
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, recipient: impl Into<String>) -> Self {
        self.recipients.push(recipient.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn body(mut self, body: EmailBody) -> Self {
        self.body = Some(body);
        self
    }

    pub fn text_body(self, body: impl Into<String>) -> Self {
        self.body(EmailBody::Text(body.into()))
    }

    pub fn html_body(self, html: impl Into<String>) -> Self {
        self.body(EmailBody::Html(html.into()))
    }

    pub fn build(self) -> Result<Email, BuildError> {
        if self.recipients.is_empty() {
            return Err(BuildError::MissingField("to"));
        }

        Ok(Email {
            from: self.from.clone(),
            to: self.recipients.join(","),
            subject: self.subject.unwrap_or_else(|| "no subject".into()),
            body: self.body,
        })
    }
}
