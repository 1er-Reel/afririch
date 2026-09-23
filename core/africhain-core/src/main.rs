use africhain_core::{Blockchain, Transaction};

fn main() {
    let mut blockchain = Blockchain::new();

    println!("AfriChain demo node started");
    println!("Genesis block created");

    blockchain.add_transaction(Transaction {
        from: "alice".to_string(),
        to: "bob".to_string(),
        amount: 50,
        nonce: 1,
    });

    blockchain.add_transaction(Transaction {
        from: "bob".to_string(),
        to: "charlie".to_string(),
        amount: 15,
        nonce: 2,
    });

    println!("Mining pending transactions...");
    blockchain.mine_pending_transactions("afri-node-01");

    println!("Blocks: {}", blockchain.chain.len());
    println!("Latest hash: {}", blockchain.chain.last().unwrap().hash);
    println!("Chain valid: {}", blockchain.is_valid());
}
