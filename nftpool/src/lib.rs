// use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
// use near_contract_standards::fungi::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::{env, ext_contract, log, near_bindgen, setup_alloc, init, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue, Promise};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::serde_json::{json, Value};
use near_sdk::serde_json::Value::String;

setup_alloc!();

const CODE :&[u8]= include_bytes!("../../res/fungible_token.wasm");
const INITIAL_BALANCE: Balance = 0;
const SOMEGAS : Gas=1000000000000;
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

    pub fn new_pool(&mut self,poolminter:AccountId, roomsize : U128)->Promise{
        let subaccount_id = format!("{}.{}", poolminter, env::current_account_id()).to_string();

        let promise= Promise::new(subaccount_id)
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(INITIAL_BALANCE)
            .deploy_contract(CODE.to_vec()).function_call(
            "new_default_meta".to_string().into_bytes(),
            json!({"owner_id":poolminter,"total_supply":roomsize.0}).to_string().into_bytes(),
            INITIAL_BALANCE,env::prepaid_gas()
        );
        return promise
    }
}

