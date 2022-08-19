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
    pub async fn send(mut self, mailer: &Mailer) -> Result<MessageId, SendError> {
        if self.from.is_none() {
            self.from.replace(mailer.from.clone());
        }

        mailer.send(self).await
    }
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct EmailBody {
    html: Option<String>,
    text: Option<String>,
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

    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        let mut body = self.body.unwrap_or_default();
        body.text = Some(text.into());
        self.body = Some(body);
        self
    }

    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        let mut body = self.body.unwrap_or_default();
        body.html = Some(html.into());
        self.body = Some(body);
        self
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
