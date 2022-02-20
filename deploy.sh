./build.sh
#
#### resetting everything
near delete nftcontract.somenewname.testnet somenewname.testnet
near delete nftpoolcontract.somenewname.testnet somenewname.testnet
near delete nfterc20contract.somenewname.testnet somenewname.testnet
#####
###### deplow section
near state somenewname.testnet
near create_account nftcontract.somenewname.testnet --masterAccount somenewname.testnet --initialBalance 10
near create_account nftpoolcontract.somenewname.testnet --masterAccount somenewname.testnet --initialBalance 10
near create_account nfterc20contract.somenewname.testnet --masterAccount somenewname.testnet --initialBalance 5
##
######
near deploy --accountId nftcontract.somenewname.testnet --wasmFile ./res/non_fungible_token.wasm --initFunction new --initArgs '{"owner_id": "nftcontract.somenewname.testnet", "name": "somename" , "symbol" : "someothername" ,"base_uri": "somenewname"}'
near deploy --accountId nftpoolcontract.somenewname.testnet --wasmFile ./res/nft_pool.wasm --initFunction new --initArgs '{"subowner" : "somenewname.testnet"}'
near deploy --accountId nfterc20contract.somenewname.testnet --wasmFile ./res/fungible_token.wasm --initFunction new_default_meta --initArgs '{"owner_id":"somenewname.testnet","total_supply":"20000000","nftcaller":"nftcontract.somenewname.testnet"}'

####
###### for calling
####
near call nftcontract.somenewname.testnet nft_mint '{"ipfs_hash":"somenamelikethis"}' --accountId somenewname.testnet
near call nftcontract.somenewname.testnet create_pool '{"pool_id":"nftpoolcontract.somenewname.testnet","roomsize":"200000000"}' --accountId somenewname.testnet --gas 300000000000000
#near call nftcontract.somenewname.testnet nft_mint '{"ipfs_hash":"somenamelikethis"}' --accountId somenewname.testnet
near call nftcontract.somenewname.testnet invite_other '{"invitee":"somenewname.testnet"}' --accountId somenewname.testnet --gas 300000000000000
#near call nftcontract.somenewname.testnet invite_other '{"invitee":"nftpoolcontract.somenewname.testnet"}' -- accountId someothername.testnet --gas 300000000000000

##near call somenewname.testnet nft_transfer '{"receiver_id":"somename.testnet","token_id":"somenamelikethis"}' --accountId somenewname.testnet --depositYocto 1

#near call somenewname123.nftpoolcontract.somenewname.testnet ft_balance_of '{"account_id":"somenewname.testnet"}' --accountId somenewname.testnet
#'somenewname20.nftpoolcontract.somenewname.testnet'



