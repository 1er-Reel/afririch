use afri_api::ApiServer;
use afri_core::{Address, Amount, Ledger, Transaction, Wallet};
use afri_net::discover_peers;
use afri_store::Store;

fn main() {
    let wallet = Wallet::new("alice");
    let tx = Transaction::new(
        Address::new("alice"),
        Address::new("bob"),
        Amount::new(50),
        1710000000,
    );

    let mut ledger = Ledger::new();
    ledger.add_transaction(tx);

    let store = Store::new("./data/ledger.json");
    store.save_ledger(&ledger).unwrap();

    println!("Wallet address: {}", wallet.address.0);
    println!("Peers: {:?}", discover_peers());
    println!("Ledger count: {}", ledger.count());

    let api = ApiServer::new("127.0.0.1", 8080);
    api.run();
}
