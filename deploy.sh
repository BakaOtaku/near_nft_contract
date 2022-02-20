./build.sh
#
#### resetting everything
near delete nftcontract.someothernewname.testnet someothernewname.testnet
near delete nftpoolcontract.someothernewname.testnet someothernewname.testnet
near delete nfterc20contract.someothernewname.testnet someothernewname.testnet
######
###### deplow section
#near state someothernewname.testnet
near create_account nftcontract.someothernewname.testnet --masterAccount someothernewname.testnet --initialBalance 10
near create_account nftpoolcontract.someothernewname.testnet --masterAccount someothernewname.testnet --initialBalance 10
near create_account nfterc20contract.someothernewname.testnet --masterAccount someothernewname.testnet --initialBalance 5
###
#######
near deploy --accountId nftcontract.someothernewname.testnet --wasmFile ./res/non_fungible_token.wasm --initFunction new --initArgs '{"owner_id": "nftcontract.someothernewname.testnet", "name": "shudanshuslovesaman" , "symbol" : "fuzuiouslovesscam" ,"base_uri": "someothernewname"}'
near deploy --accountId nftpoolcontract.someothernewname.testnet --wasmFile ./res/nft_pool.wasm --initFunction new --initArgs '{"subowner" : "someothernewname.testnet"}'
near deploy --accountId nfterc20contract.someothernewname.testnet --wasmFile ./res/fungible_token.wasm --initFunction new_default_meta --initArgs '{"owner_id":"someothernewname.testnet","total_supply":"20000000","nftcaller":"nftcontract.someothernewname.testnet"}'
#
#####
####### for calling
#####
#near call nftcontract.someothernewname.testnet create_pool '{"pool_id":"nftpoolcontract.someothernewname.testnet","roomsize":"200000000"}' --accountId someothernewname.testnet --gas 300000000000000
near call nftcontract.someothernewname.testnet nft_mint '{"ipfs_hash":"https://avatars.githubusercontent.com/u/42795731?v=4"}' --accountId someothernewname.testnet --gas 300000000000000
#near call nftcontract.someothernewname.testnet nft_mint '{"ipfs_hash":"https://avatars.githubusercontent.com/u/42104907?v=4"}' --accountId someothernewname.testnet
near call nftcontract.someothernewname.testnet invite_other '{"invitee":"someothernewname.testnet"}' --accountId someothernewname.testnet --gas 300000000000000
#near call nftcontract.someothernewname.testnet invite_other '{"invitee":"someothernewname.testnet"}' -- accountId someothername.testnet --gas 300000000000000

#near call nftcontract.someothernewname.testnet nft_transfer '{"receiver_id":"testing2someothername.testnet","token_id":"1"}' --accountId someothernewname.testnet --depositYocto 1

#near call someothernewname123.nftpoolcontract.someothernewname.testnet ft_balance_of '{"account_id":"someothernewname.testnet"}' --accountId someothernewname.testnet
#'someothernewname20.nftpoolcontract.someothernewname.testnet'

near call nftcontract.someothernewname.testnet invite_left '{"account_id":"someothernewname.testnet"}' --accountId someothernewname.testnet

