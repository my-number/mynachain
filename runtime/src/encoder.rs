mod types;

fn main() {
    println!("encoder");
    let tx = types::TxCreateAccount {
        cert: vec![1,2,3,4,5,6,7,8],
        nonce: 0
    }
    let tbs = types::Tx::CreateAccount(tx);
    let encoded = tbs.encode();
    println!("{:?}", encoded);
}
