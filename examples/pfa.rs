use amplify::hex::FromHex;
use bp::{CompressedPk, Txid};
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::contract::{FilterIncludeAll, FungibleAllocation, IssuerWrapper};
use rgbstd::invoice::Precision;
use rgbstd::persistence::Stock;
use rgbstd::stl::{AssetSpec, ContractTerms, RicardianContract};
use rgbstd::{Amount, ChainNet, GenesisSeal};
use schemata::dumb::NoResolver;
use schemata::PermissionedFungibleAsset;

fn main() {
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary = GenesisSeal::new_random(beneficiary_txid, 1);

    let spec = AssetSpec::new("TEST", "Test asset", Precision::CentiMicro);

    let terms = ContractTerms {
        text: RicardianContract::default(),
        media: None,
    };

    let issued_supply = Amount::from(100000u64);

    let pubkey = CompressedPk::from_byte_array([
        2, 199, 163, 211, 116, 75, 108, 119, 241, 66, 54, 236, 233, 189, 142, 108, 37, 135, 56,
        128, 200, 176, 199, 9, 117, 132, 72, 200, 167, 185, 4, 64, 53,
    ])
    .unwrap();

    let mut stock = Stock::in_memory();
    let kit = Kit::load_file("schemata/PermissionedFungibleAsset.rgb")
        .unwrap()
        .validate()
        .unwrap();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock
        .contract_builder(
            "ssi:anonymous",
            PermissionedFungibleAsset::schema().schema_id(),
            ChainNet::BitcoinTestnet4,
        )
        .unwrap()
        .add_global_state("spec", spec)
        .expect("invalid spec")
        .add_global_state("terms", terms)
        .expect("invalid contract terms")
        .add_global_state("issuedSupply", issued_supply)
        .expect("invalid issued supply")
        .add_fungible_state("assetOwner", beneficiary, 100000u64)
        .expect("invalid fungible state")
        .add_global_state("pubkey", pubkey)
        .expect("invalid pubkey")
        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract
        .save_file("test/pfa-example.rgb")
        .expect("unable to save contract");
    contract
        .save_armored("test/pfa-example.rgba")
        .expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state from the stock:
    let contract = stock
        .contract_wrapper::<PermissionedFungibleAsset>(contract_id)
        .unwrap();
    let allocations = contract.allocations(&FilterIncludeAll);
    eprintln!("\nThe issued contract:");
    eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

    for FungibleAllocation {
        seal,
        state,
        witness,
        ..
    } in allocations
    {
        let witness = witness
            .as_ref()
            .map(Txid::to_string)
            .unwrap_or("~".to_owned());
        eprintln!("amount={}, owner={seal}, witness={witness}", state.value());
    }
    eprintln!("totalSupply={}", contract.total_issued_supply().value());
}
