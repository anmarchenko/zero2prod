use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        EmailClient {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }
    pub async fn send_email(
        &mut self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
