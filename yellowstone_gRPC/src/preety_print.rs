use crate::types::{SolanaAccount, SolanaTransaction, TransactionInstruction};
use std::fmt;

impl fmt::Display for TransactionInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    TransactionInstruction:")?;
        writeln!(f, "      program_id: {}", self.program_id)?;
        writeln!(f, "      accounts: [")?;
        for acc in &self.accounts {
            writeln!(f, "        \"{}\",", acc)?;
        }
        writeln!(f, "      ]")?;
        writeln!(f, "      data: {}", self.data)?;
        Ok(())
    }
}

impl fmt::Display for SolanaTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SolanaTransaction:")?;
        writeln!(f, "  signature: {}", self.signature)?;
        writeln!(f, "  slot: {}", self.slot)?;
        writeln!(f, "  is_vote: {}", self.is_vote)?;
        writeln!(f, "  index: {}", self.index)?;
        writeln!(f, "  success: {}", self.success)?;
        writeln!(f, "  fee: {:?}", self.fee)?;
        writeln!(
            f,
            "  compute_units_consumed: {:?}",
            self.compute_units_consumed
        )?;

        writeln!(f, "  instructions: [")?;
        for instr in &self.instructions {
            writeln!(f, "{}", instr)?;
        }
        writeln!(f, "  ]")?;

        writeln!(f, "  account_keys: [")?;
        for key in &self.account_keys {
            writeln!(f, "    \"{}\",", key)?;
        }
        writeln!(f, "  ]")?;

        writeln!(f, "  log_messages: [")?;
        for log in &self.log_messages {
            writeln!(f, "    \"{}\",", log)?;
        }
        writeln!(f, "  ]")?;

        writeln!(f, "  pre_balances: {:?}", self.pre_balances)?;
        writeln!(f, "  post_balances: {:?}", self.post_balances)?;
        writeln!(f, "  timestamp: {}", self.timestamp)?;
        Ok(())
    }
}

impl fmt::Display for SolanaAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SolanaAccount:")?;
        writeln!(f, "  pubkey: {}", self.pubkey)?;
        writeln!(f, "  lamports: {}", self.lamports)?;
        writeln!(f, "  owner: {}", self.owner)?;
        writeln!(f, "  executable: {}", self.executable)?;
        writeln!(f, "  rent_epoch: {}", self.rent_epoch)?;
        writeln!(f, "  data: {}", self.data)?;
        writeln!(f, "  write_version: {}", self.write_version)?;
        writeln!(f, "  slot: {}", self.slot)?;
        writeln!(
            f,
            "  txn_signature: {}",
            self.txn_signature.as_deref().unwrap_or("None")
        )?;
        writeln!(f, "  timestamp: {}", self.timestamp)?;
        Ok(())
    }
}
