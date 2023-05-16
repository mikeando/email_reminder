use anyhow::{Context, Result};
use clap::Parser;
use dotenv::dotenv;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::PoolConfig;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

struct ReminderMessage {
    recipient: Mailbox,
    sender: Mailbox,
    subject: String,
    body: String,

    server: String,
    password: String,
}

impl ReminderMessage {
    pub fn send(&self) -> Result<()> {
        let email = Message::builder()
            .to(self.recipient.clone())
            .from(self.sender.clone())
            .subject(&self.subject)
            .body(self.body.clone())
            .context("unable to build email")?;

        // Create TLS transport on port 587 with STARTTLS
        let mailer = SmtpTransport::starttls_relay(&self.server)
            .context("unable to create mailer")?
            // Add credentials for authentication
            .credentials(Credentials::new(
                format!(
                    "{}@{}",
                    self.sender.email.user(),
                    self.sender.email.domain()
                ),
                self.password.clone(),
            ))
            // Configure expected authentication mechanism
            .authentication(vec![Mechanism::Plain])
            // Connection pool settings
            .pool_config(PoolConfig::new().max_size(20))
            .build();

        let _result = mailer.send(&email).context("unable to send email")?;
        Ok(())
    }
}

/// Simple program to send an email
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Recipient
    #[arg(short, long)]
    recipient: String,

    /// Message subject
    #[arg(short, long)]
    subject: String,

    /// Message body
    #[arg(short, long)]
    body: String,
}

fn main() {
    dotenv().ok();
    let args = Args::parse();

    let sender_mbox: Mailbox = env::var("SMTP_MAIL_USER")
        .context("missing SMTP_MAIL_USER variable")
        .unwrap()
        .parse()
        .context("unable to parse SMTP_MAIL_USER variable")
        .unwrap();
    let recipient_mbox: Mailbox = args
        .recipient
        .parse()
        .context("unable to parse recipient")
        .unwrap();
    let server = env::var("SMTP_MAIL_SERVER")
        .context("missing SMTP_MAIL_SERVER variable")
        .unwrap();
    let password = env::var("SMTP_MAIL_PASSWORD")
        .context("missing SMTP_MAIL_PASSWORD variable")
        .unwrap();

    let mesg = ReminderMessage {
        recipient: recipient_mbox,
        sender: sender_mbox,
        subject: args.subject,
        body: args.body,
        server,
        password,
    };

    mesg.send().unwrap();
}
