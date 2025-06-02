use amplify::hex::FromHex;
use bp::Txid;
use rgbstd::containers::{ConsignmentExt, FileContent, Kit};
use rgbstd::contract::{FilterIncludeAll, FungibleAllocation, IssuerWrapper};
use rgbstd::invoice::Precision;
use rgbstd::persistence::Stock;
use rgbstd::stl::{AssetSpec, ContractTerms, OpidRejectUrl, RicardianContract};
use rgbstd::{Amount, ChainNet, GenesisSeal};
use schemata::dumb::NoResolver;
use schemata::InflatableFungibleAsset;

fn main() {
    let beneficiary_txid =
        Txid::from_hex("14295d5bb1a191cdb6286dc0944df938421e3dfcbf0811353ccac4100c2068c5").unwrap();
    let beneficiary_1 = GenesisSeal::new_random(beneficiary_txid, 1);
    let beneficiary_2 = GenesisSeal::new_random(beneficiary_txid, 2);
    let beneficiary_3 = GenesisSeal::new_random(beneficiary_txid, 3);

    let spec = AssetSpec::new("TEST", "Test asset", Precision::CentiMicro);

    let terms = ContractTerms {
        text: RicardianContract::default(),
        media: None,
    };

    let issued_supply = Amount::from(100000u64);

    let max_supply = Amount::from(150000u64);

    let opid_reject_url = OpidRejectUrl::from("example.xyz/opidReject");

    let mut stock = Stock::in_memory();
    let kit = Kit::load_file("schemata/InflatableFungibleAsset.rgb")
        .unwrap()
        .validate()
        .unwrap();
    stock.import_kit(kit).expect("invalid issuer kit");

    let contract = stock
        .contract_builder(
            "ssi:anonymous",
            InflatableFungibleAsset::schema().schema_id(),
            ChainNet::BitcoinTestnet4,
        )
        .unwrap()
        .add_global_state("spec", spec)
        .expect("invalid spec")
        .add_global_state("terms", terms)
        .expect("invalid contract terms")
        .add_global_state("issuedSupply", issued_supply)
        .expect("invalid issued supply")
        .add_global_state("maxSupply", max_supply)
        .expect("invalid max supply")
        .add_global_state("opidRejectUrl", opid_reject_url)
        .expect("invalid opid reject url")
        .add_fungible_state("assetOwner", beneficiary_1, issued_supply.value())
        .expect("invalid fungible state")
        .add_fungible_state(
            "inflationAllowance",
            beneficiary_2,
            max_supply.value() - issued_supply.value(),
        )
        .expect("invalid fungible state")
        .add_rights("replaceRight", beneficiary_3)
        .expect("invalid void state")
        .issue_contract()
        .expect("contract doesn't fit schema requirements");

    let contract_id = contract.contract_id();

    eprintln!("{contract}");
    contract
        .save_file("test/ifa-example.rgb")
        .expect("unable to save contract");
    contract
        .save_armored("test/ifa-example.rgba")
        .expect("unable to save armored contract");

    stock.import_contract(contract, NoResolver).unwrap();

    // Reading contract state from the stock:
    let contract = stock
        .contract_wrapper::<InflatableFungibleAsset>(contract_id)
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
