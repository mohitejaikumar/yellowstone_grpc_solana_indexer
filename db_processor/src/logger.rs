use yellowstone_gRPC::types::IndexEvent;

pub fn logging(message: IndexEvent) {
    match message {
        IndexEvent::Transaction(transaction) => {
            println!("{}", transaction);
        }
        IndexEvent::Account(account) => {
            println!("{}", account);
        }
        IndexEvent::Slot(slot) => {
            println!("{}", slot);
        }
        IndexEvent::Block(block) => {
            println!("{}", block);
        }
    }
}
