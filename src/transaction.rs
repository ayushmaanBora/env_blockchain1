use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskStatus {
    PendingValidation,
    Validated,
    Rejected,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub task: String,
    pub proof_metadata: String,
    pub status: TaskStatus, // Replaced 'verified: bool'
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, task: String, proof_metadata: String) -> Self {
        Self {
            sender,
            receiver,
            amount,
            task,
            proof_metadata,
            status: TaskStatus::PendingValidation, // Default to pending
        }
    }
}