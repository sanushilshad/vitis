// use crate::{configuration::EmailClientConfig, domain::EmailObject};
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::SinglePart,
    transport::smtp::{PoolConfig, authentication::Credentials},
};

use crate::{configuration::EmailClientConfig, email::EmailObject};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use uuid::Uuid;
// #[derive(Clone)]
// pub struct EmailClient {
//     sender: EmailObject,
//     pub mailer: AsyncSmtpTransport<Tokio1Executor>,
// }
// impl EmailClient {
//     #[tracing::instrument]
//     pub fn new(email_config: EmailClientSettings) -> Result<Self, Box<dyn std::error::Error>> {
//         let sender = email_config
//             .sender()
//             .expect("Invalid sender email address.");
//         let smtp_credentials = Credentials::new(
//             email_config.username,
//             email_config.password.expose_secret().to_string(),
//         );
//         tracing::info!("Establishing  connection to the SMTP server.");
//         let mailer: AsyncSmtpTransport<Tokio1Executor> =
//             AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.base_url)?
//                 .credentials(smtp_credentials)
//                 .pool_config(
//                     PoolConfig::new()
//                         .min_idle(3)
//                         .max_size(10)
//                         .idle_timeout(Duration::new(300, 0)),
//                 )
//                 .build();

//         tracing::info!("SMTP connection created succuessfully");
//         Ok(Self { sender, mailer })
//     }

//     pub async fn send_text_email(
//         &self,
//         // mailer: &AsyncSmtpTransport<Tokio1Executor>,
//         to: &str,
//         subject: &str,
//         body: String,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         tracing::info!("SMTP Parameters: {:?}", self.mailer);
//         let email = Message::builder()
//             .from(self.sender.as_ref().parse()?)
//             .to(to.parse()?)
//             .subject(subject)
//             .body(body.to_string())?;

//         tracing::info!("Sending Email");
//         self.mailer.send(email).await?;
//         tracing::info!("Mail Send Successfully");
//         Ok(())
//     }

//     pub async fn send_html_email(
//         &self,
//         // mailer: &AsyncSmtpTransport<Tokio1Executor>,
//         to: &str,
//         subject: &str,
//         body: String,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         tracing::info!("SMTP Parameters: {:?}", self.mailer);
//         Ok(())
//     }
// }

#[async_trait]
pub trait GenericEmailService: Send + Sync {
    async fn send_text_email(
        &self,
        to: &EmailObject,
        cc: &Option<Vec<EmailObject>>,
        subject: &str,
        body: String,
        message_id: Option<String>,
        in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn send_html_email(
        &self,
        to: &EmailObject,
        cc: &Option<Vec<EmailObject>>,
        subject: &str,
        body: String,
        message_id: Option<String>,
        in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SmtpEmailClient {
    pub sender: EmailObject,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

#[async_trait]
impl GenericEmailService for DummyEmailClient {
    async fn send_text_email(
        &self,
        _to: &EmailObject,
        _cc: &Option<Vec<EmailObject>>,
        _subject: &str,
        _body: String,
        _message_id: Option<String>,
        _in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn send_html_email(
        &self,
        _to: &EmailObject,
        _cc: &Option<Vec<EmailObject>>,
        _subject: &str,
        _body: String,
        _message_id: Option<String>,
        _in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl SmtpEmailClient {
    #[tracing::instrument]
    // pub fn new(email_config: &EmailClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
    pub fn new_personal(
        sender: &EmailObject,
        key: SecretString,
        base_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let sender = EmailObject::new(sender.to_string());
        let smtp_credentials = Credentials::new(sender.to_string(), key.expose_secret().to_owned());
        tracing::info!("Establishing  connection to the SMTP server.");
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(base_url)?
                .credentials(smtp_credentials)
                .build();

        tracing::info!("SMTP connection created succuessfully");
        Ok(Self { sender, mailer })
    }

    pub fn generate_message_id(&self, domain: &str) -> String {
        format!("<{}@{}>", Uuid::new_v4(), domain)
    }

    #[tracing::instrument]
    pub fn new(email_config: &EmailClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let sender = email_config
            .sender()
            .expect("Invalid sender email address.");
        let smtp_credentials = Credentials::new(
            email_config.username.to_string(),
            email_config.password.expose_secret().to_string(),
        );
        tracing::info!("Establishing  connection to the SMTP server.");
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.base_url)?
                .credentials(smtp_credentials)
                .pool_config(
                    PoolConfig::new()
                        .min_idle(3)
                        .max_size(10)
                        .idle_timeout(Duration::new(30000, 0)),
                )
                .build();

        // tracing::info!("SMTP connection created succuessfully");
        Ok(Self { sender, mailer })
    }
}

#[async_trait]
impl GenericEmailService for SmtpEmailClient {
    async fn send_text_email(
        &self,
        to: &EmailObject,
        cc: &Option<Vec<EmailObject>>,
        subject: &str,
        body: String,
        message_id: Option<String>,
        in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("SMTP Parameters: {:?}", self.mailer);
        let mut builder = Message::builder()
            .from(self.sender.as_ref().parse()?)
            .to(to.get().parse()?)
            .message_id(message_id);
        if let Some(cc_list) = cc {
            for cc_email in cc_list {
                builder = builder.cc(cc_email.get().parse()?);
            }
        }
        if let Some(mid) = &in_reply_to {
            builder = builder
                .in_reply_to(mid.to_owned())
                .references(mid.to_owned());
        };
        let email = builder.subject(subject).body(body.to_string())?;

        tracing::info!("Sending Email");
        self.mailer.send(email).await?;
        tracing::info!("Mail Send Successfully");
        Ok(())
    }

    async fn send_html_email(
        &self,
        // mailer: &AsyncSmtpTransport<Tokio1Executor>,
        to: &EmailObject,
        cc: &Option<Vec<EmailObject>>,
        subject: &str,
        body: String,
        message_id: Option<String>,
        in_reply_to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("SMTP Parameters: {:?}", self.mailer);
        let mut builder = Message::builder()
            .from(self.sender.as_ref().parse()?)
            .to(to.get().parse()?)
            .message_id(message_id);

        if let Some(cc_list) = cc {
            for cc_email in cc_list {
                builder = builder.cc(cc_email.get().parse()?);
            }
        }
        if let Some(mid) = &in_reply_to {
            builder = builder
                .in_reply_to(mid.to_owned())
                .references(mid.to_owned());
        };

        let email = builder
            .subject(subject)
            .singlepart(SinglePart::html(body))?;

        tracing::info!("Sending HTML Email");
        self.mailer.send(email).await.map_err(|e| {
            tracing::error!("{}", e);
            e
        })?;
        tracing::info!("HTML Email Sent Successfully");
        Ok(())
    }
}

pub struct DummyEmailClient {}

impl DummyEmailClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Establishing dummy connection to the SMTP server.");
        tracing::info!("Dummy SMTP connection created succuessfully");
        Ok(Self {})
    }
}
