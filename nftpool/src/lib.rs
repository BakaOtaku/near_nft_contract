// use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
// use near_contract_standards::fungi::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::{env, ext_contract, log, near_bindgen, setup_alloc, init, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue, Promise};
use near_sdk::collections::{LazyOption, LookupMap};
// use near_sdk::PromiseOrValue::Promise;
use near_sdk::serde_json::{json, Value};
use near_sdk::serde_json::Value::String;

setup_alloc!();

const CODE :&[u8]= include_bytes!("../../res/fungible_token.wasm");
const NO_DEPOSIT: Balance = 5226110000000000000000000;
const BASE_GAS: Gas = 5_000_000_000_000;
const PROMISE_CALL: Gas = 5_000_000_000_000;
const GAS_FOR_NFT_ON_APPROVE: Gas = BASE_GAS + PROMISE_CALL;

trait DeployPool {
    fn new_pool(&mut self, poolname:AccountId, owner_id:AccountId,roomsize :U128) -> PromiseOrValue<AccountId>;
}

// #[ext_contract(ext_erc)]
// pub trait Intializer{
//     fn new_init()->PromiseOrValue<String>;
// }




#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Pool {
    owner: AccountId,
    subowner :AccountId,
    token: LookupMap<AccountId,AccountId>
}



#[near_bindgen]
impl Pool{

    #[init]
    pub fn new(subowner : AccountId)->Self{
        Self{
            owner:env::predecessor_account_id(),
            subowner,
            token: LookupMap::new(b"a"),
        }
    }

    pub fn change_owner(&mut self,subowner:AccountId) ->bool{
        assert_eq!(env::predecessor_account_id(), self.owner);
        self.subowner=subowner;
        return true;
    }

    // pub fn new_pool(&mut self,poolminter:AccountId, roomsize : U128)->Promise{
    //     assert!(env::predecessor_account_id(), self.subowner);
    //     let subaccount_id = format!("{}.{}", poolminter, env::current_account_id()).to_string();
    //     let subaccout_copy= subaccount_id.clone();
    //     let promise= Promise::new(subaccount_id)
    //         .create_account()
    //         .add_full_access_key(env::signer_account_pk())
    //         .transfer(INITIAL_BALANCE)
    //         .deploy_contract(CODE.to_vec()).function_call(
    //         "new_default_meta".to_string().into_bytes(),
    //         json!({"owner_id":poolminter,"total_supply":roomsize.0}).to_string().into_bytes(),
    //         INITIAL_BALANCE,env::prepaid_gas()
    //     );
    //
    //     self.token.insert(&poolminter, &subaccout_copy);
    //
    //     return promise
    // }
}
#[near_bindgen]
impl DeployPool for Pool{
    fn new_pool(&mut self, poolname: AccountId, owner_id :AccountId, roomsize: U128) -> PromiseOrValue<AccountId> {
        // assert_eq!(env::predecessor_account_id(), self.subowner.to_string());
        // let subaccount_id = AccountId::new_unchecked(
        //     format!("{}.{}", prefix, env::current_account_id())
        // );
        // let somename = ValidAccountId::try_from(poolminter.clone()).unwrap();
        log!("{}",env::prepaid_gas().to_string());
        let subaccount_id = format!("{}.{}", poolname, env::current_account_id()).to_string();
        let stuff =Promise::new(subaccount_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(NO_DEPOSIT)
            .deploy_contract(CODE.to_vec());

        env::log("this was here".to_string().as_bytes());
        log!("creating nft pool for erc20");
        // let callback = Promise::new(
        //     env::current_account_id(), // the recipient of this ActionReceipt (&self)
        // )

        let otherpromise=Promise::new(subaccount_id.clone()).function_call(
            b"new_default_meta".to_vec(),
            json!({"owner_id":owner_id.clone(),"total_supply":roomsize,"nftcaller":"nftcontract.someothernewname.testnet"}).to_string().into_bytes(),
0,
            5_000_000_000_000
        );


        stuff.then(otherpromise);

        //
        // env::promise_return(otherpromise);

        return PromiseOrValue::Value(subaccount_id);
    }
}

