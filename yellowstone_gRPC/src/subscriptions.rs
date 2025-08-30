use std::collections::HashMap;

use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterTransactions,
};

pub struct Subscriptions;

impl Subscriptions {
    pub fn create_defi_subscription() -> SubscribeRequest {
        let mut accounts = HashMap::new();
        accounts.insert(
            "defi_accounts".to_string(),
            SubscribeRequestFilterAccounts {
                account: vec![],
                owner: vec![
                    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                    "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
                    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                ],
                filters: vec![],
                nonempty_txn_signature: None,
            },
        );

        let mut transactions = HashMap::new();
        transactions.insert(
            "defi_transactions".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec![
                    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                    "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
                    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                ],
                account_exclude: vec![],
                account_required: vec![],
            },
        );
        SubscribeRequest {
            accounts,
            transactions,
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            entry: HashMap::new(),
            commitment: Some(CommitmentLevel::Confirmed as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
            slots: HashMap::new(),
            transactions_status: HashMap::new(),
        }
    }
}
