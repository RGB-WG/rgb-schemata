use amplify::hex::FromHex;
use bp::dbc::Method;
use bp::{Outpoint, Txid};
use ifaces::Rgb20;
use rgbstd::containers::{FileContent, Kit};
use rgbstd::interface::FilterIncludeAll;
use rgbstd::invoice::Precision;
use rgbstd::persistence::{MemIndex, MemStash, MemState, Stock};
use schemata::dumb::DumbResolver;
use schemata::InflatableAsset;

#[rustfmt::skip]
fn main() {
    let beneficiary_txid = 
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = Outpoint::new(beneficiary_txid, 1);

    let contract = Rgb20::testnet::<InflatableAsset>("ssi:anonymous","TEST", "Test asset", None, Precision::CentiMicro)
        .expect("invalid contract data")
        .allocate(Method::TapretFirst, beneficiary, 1_000_000_000_00u64)
        .expect("invalid allocations")
        .allow_inflation(Method::TapretFirst, beneficiary, 1_000_000_000_00u64)
        .expect("invalid inflation allowance")
        .issue_contract()
        .expect("invalid contract data");

    let contract_id = contract.contract_id();

    // eprintln!("{contract}");
    contract.save_file("test/ia-rgb20-example.rgb").expect("unable to save contract");
    contract.save_armored("test/ia-rgb20-example.rgba").expect("unable to save armored contract");

    let kit = Kit::load_file("schemata/InflatableAssets.rgb").unwrap().validate().unwrap();

    // Let's create some stock - an in-memory stash and inventory around it:
    let mut stock = Stock::<MemStash, MemState, MemIndex>::default();
    stock.import_kit(kit).expect("invalid issuer kit");
    stock.import_contract(contract, &mut DumbResolver).unwrap();

    // Reading contract state through the interface from the stock:
    let contract = stock.contract_iface_class::<Rgb20>(contract_id).unwrap();
    let contract = Rgb20::from(contract);
    eprintln!("\nThe issued contract data:");
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

    
    for global in &contract.iface.global_state {
        if let Ok(values) = contract.global(global.name.clone()) {
            for val in values {
                println!("  {} := {}", global.name, val);
            }
        }
    }
    for state in &contract.iface.assignments {
        println!("  {}:", state.name);
        if let Ok(owned_state) = contract.fungible("assetOwner", &FilterIncludeAll) {
            for allocation in owned_state {
                println!(
                    "    value={}, utxo={}, witness={}",
                    allocation.state.value(),
                    allocation.seal,
                    allocation.witness,
                );
            }
        }
    }
    eprintln!("totalSupply={:?}", contract.total_supply());
}
