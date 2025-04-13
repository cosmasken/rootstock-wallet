pub struct Web3Client;

impl Web3Client {
    pub fn new() -> Self {
        Web3Client
    }

    pub fn fetch_balance(&self, address: &str) -> String {
        format!("Dummy balance for {}", address)
    }

    pub fn send_transaction(&self, sender_pk: &str, receiver: &str, amount: &str) -> String {
        format!(
            "Transaction sent: {} -> {} : {}",
            sender_pk, receiver, amount
        )
    }
}
