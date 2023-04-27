
import {
  storeCode, instantiateContract, executeContract, queryStakingDelegations,
  queryWasmContract, queryAddressBalance, queryStaking, queryStakingParameters, sendCoin
} from "./common";
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { parseCoins, coins, coin } from "@cosmjs/stargate";

async function main(): Promise<void> {


  const LCD_ENDPOINT = "http://127.0.0.1:1317/";
  const RPC_ENDPOINT = "http://127.0.0.1:26657";
  const mnemonic = "symbol animal admit garment jeans climb drum net drill advice novel feed pride machine fence trim embark melt object sudden increase dish stay liar";
  const mnemonic2 = "violin burst wear grape artefact vessel purpose edge engage despair stumble gasp river wheat develop wheat trophy confirm lottery tag trash ice state remain";
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix: "sei", });
  const [account] = await wallet.getAccounts();
  const wallet2 = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic2, { prefix: "sei", });
  const [account2] = await wallet2.getAccounts();


  let hubCodeId = 253;
  let rewardCodeId = 254;
  let bSeiTokenCodeId = 255;
  let rewardsDispatcherCodeId = 256;
  let validatorsRegistryCodeId = 257;
  let stSeiTokenCodeId = 258;

  let hubAddress = "sei1nsugwwd4nsz0xny7g2y6rgmnuxghv6chcxjhzphn956l8tg7j8zsspg8z5";
  let rewardAddress = "sei1s83x9wmhwjw8venj3y4jamrap0mk42ngda5z2020dmqavvrfazys5wcale";
  let bSeiTokenAddress = "sei1c0a4h6c8qh6j0wmk998q590af9llth90d7dzlkk97naqzsdtepass3srvp";
  let rewardsDispatcherAddress = "sei1x7hl9rs2pcr23dtf6dncmjyj803wt2udusn2gkkc58tw27n2088swnvatm";
  let validatorsRegistryAddress = "sei1snccckkpjw2awl9gycpcsayl9ccesf48x8dqwarg5mzsskmkhz6spvghxw";
  let stSeiTokenAddress = "sei1hkchpay86jhdwtkjkth6pevly63kp8p347wmyf5nhcf9sk2rld9sv0gcav";

  const stable_coin_denom = "factory/sei1h3ukufh4lhacftdf6kyxzum4p86rcnel35v4jk/USDT";

  // hubCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_hub.wasm")
  // rewardCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_reward.wasm")
  // bSeiTokenCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_token_bsei.wasm")
  // rewardsDispatcherCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_rewards_dispatcher.wasm")
  // validatorsRegistryCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_validators_registry.wasm")
  // stSeiTokenCodeId = await storeCode(RPC_ENDPOINT, wallet, "../artifacts/basset_sei_token_stsei.wasm")

  // console.log()

  // hubAddress = await instantiateContract(RPC_ENDPOINT, wallet, hubCodeId,
  //   {
  //     epoch_period: 30, 
  //     er_threshold: "1.0", 
  //     peg_recovery_fee: "0", 
  //     reward_denom: stable_coin_denom, 
  //     unbonding_period: 120, 
  //     underlying_coin_denom: "usei", 
  //     validator: "seivaloper1h3ukufh4lhacftdf6kyxzum4p86rcnel0mamnx"   //local node validator address
  //   }, parseCoins("") , "lido sei hub")

  // rewardAddress = await instantiateContract(RPC_ENDPOINT, wallet, rewardCodeId,
  //   { 
  //     owner : account.address,
  //     hub_contract: hubAddress, 
  //     reward_denom: stable_coin_denom, 
  //   }, parseCoins(""), "sei reward")

  // bSeiTokenAddress = await instantiateContract(RPC_ENDPOINT, wallet, bSeiTokenCodeId,
  //   {decimals: 6, hub_contract: hubAddress, initial_balances: [],
  //     name: "bsei", symbol: "BSEI",
  //     mint: {minter: hubAddress, cap: null}}, parseCoins(""),"bond sei")

  // rewardsDispatcherAddress = await instantiateContract(RPC_ENDPOINT, wallet, rewardsDispatcherCodeId,
  //   {
  //     lido_fee_address: account.address,
  //     lido_fee_rate: "0.05", 
  //     hub_contract: hubAddress, 
  //     bsei_reward_contract: rewardAddress,
  //     stsei_reward_denom: "usei", 
  //     bsei_reward_denom: stable_coin_denom, 
  //   }, parseCoins(""), "reward dispatcher")

  // validatorsRegistryAddress = await instantiateContract(RPC_ENDPOINT, wallet, validatorsRegistryCodeId,
  //   {
  //     hub_contract: hubAddress,
  //     registry: [{active: true,
  //       address: "seivaloper1h3ukufh4lhacftdf6kyxzum4p86rcnel0mamnx",    //env 192.168.2.66 validator
  //       total_delegated: "0"}]}, parseCoins(""), "validator registery")

  // stSeiTokenAddress = await instantiateContract(RPC_ENDPOINT, wallet, stSeiTokenCodeId,
  //   {
  //     decimals: 6, 
  //     hub_contract: hubAddress, 
  //     initial_balances: [],
  //     name: "stsei", symbol: "STSEI",
  //     mint: {minter: hubAddress, cap: null}}, parseCoins(""), "staking sei")

  // console.log()

  // console.log(`HUB_CONTRACT = ${hubAddress}`)
  // console.log(`REWARD_CONTRACT = ${rewardAddress}`)
  // console.log(`BSEI_TOKEN_CONTRACT = ${bSeiTokenAddress}`)
  // console.log(`REWARDS_DISPATCHER_CONTRACT = ${rewardsDispatcherAddress}`)
  // console.log(`VALIDATORS_REGISTRY_CONTRACT = ${validatorsRegistryAddress}`)
  // console.log(`STSEI_TOKEN_CONTRACT = ${stSeiTokenAddress}`)

  // //////////////////////////////////////configure contracts///////////////////////////////////////////
  // //////////////////////////////////////must behind deploy moneymarket contract///////////////////////
  // console.log("Updating hub's config...")
  // await executeContract(RPC_ENDPOINT, wallet, hubAddress, {
  //   update_config: {
  //     bsei_token_contract: bSeiTokenAddress, 
  //     stsei_token_contract: stSeiTokenAddress,
  //     rewards_dispatcher_contract: rewardsDispatcherAddress,
  //     validators_registry_contract: validatorsRegistryAddress,
  //     rewards_contract: rewardAddress,
  //   }}, "", parseCoins(""))
  // console.log("Updating hub's config end")


  //======================deployed contracts，change creater to update_global_index=======================================//  
  //change creater，
  // await executeContract(RPC_ENDPOINT, wallet, hubAddress, 
  // {
  //   update_config: {
  //     owner : "sei1xm3mccak0yjfts96jszdldxka6xkw00ywv6au0"
  //   }
  // }, "", parseCoins("") )
  // console.log("transfer owener ok.")



  // //just a few simple tests to make sure the contracts are not failing
  // //for more accurate tests we must use integration-tests repo
  await executeContract(RPC_ENDPOINT, wallet, hubAddress, { bond_for_st_sei: {} }, "", parseCoins("1000000usei"))
  console.log("test hubaddress bond for stSei ok...")

  await executeContract(RPC_ENDPOINT, wallet, hubAddress, { bond_for_st_sei: {} }, "", parseCoins("1000000usei"))
  console.log("test hubaddress bond for stSei ok...")

  await executeContract(RPC_ENDPOINT, wallet, hubAddress, { bond: {} }, "", parseCoins("10000000usei"))
  console.log("test hubaddress bond for bSei ok...")
  console.log("query bond sei balance:")
  await queryWasmContract(RPC_ENDPOINT, wallet, bSeiTokenAddress, { balance: { address: account.address } })


  await executeContract(RPC_ENDPOINT, wallet, bSeiTokenAddress, {
    send: {
      contract: hubAddress, amount: "500000",
      msg: Buffer.from(JSON.stringify({ "unbond": {} })).toString('base64')
    }
  }, "", parseCoins(""))
  console.log("send bseiTokenAddress unbond for bSei ok...")

  console.log("query hub contract balance:")
  await queryAddressBalance(LCD_ENDPOINT, hubAddress, stable_coin_denom);

  console.log("query withdraw able unbonded:")
  await queryWasmContract(RPC_ENDPOINT, wallet, hubAddress, { withdrawable_unbonded: { address: account.address } })

  console.log("query staking pool:")
  await queryStaking(LCD_ENDPOINT);


  console.log("query current batch:")
  await queryWasmContract(RPC_ENDPOINT, wallet, hubAddress, { current_batch: {} })


  console.log("query unbond request:")
  await queryWasmContract(RPC_ENDPOINT, wallet, hubAddress, { unbond_requests: { address: account.address } })


  console.log("query staking parameter:")
  await queryStakingParameters(LCD_ENDPOINT);

  // console.log("query delegations list:")
  // await queryStakingDelegations(LCD_ENDPOINT, account.address, "seivaloper1wukzl3ppckudkkny744k4kmrw0p0l6h98sm43s");

  console.log("withdraw able unbonded:")
  let withdrawRet = await executeContract(RPC_ENDPOINT, wallet, hubAddress, { withdraw_unbonded: {} }, "", parseCoins(""))
  console.log("withdraw able unbonded ok")

  console.log("query hub config:")
  await queryWasmContract(RPC_ENDPOINT, wallet, hubAddress, { config: {} })

  await queryAddressBalance(LCD_ENDPOINT, account.address, "")

  await executeContract(RPC_ENDPOINT, wallet, hubAddress, {update_global_index : {}}, "",
  coins(100000000, stable_coin_denom))
  console.log("update_reward_basset ok")

  console.log("query accured reward:")
  await queryWasmContract(RPC_ENDPOINT, wallet, rewardAddress,
    {
      accrued_rewards:
      {
        address: account.address
      }
    })

  console.log("query rewards balance:")
  await queryAddressBalance(LCD_ENDPOINT, rewardAddress, stable_coin_denom)

  // console.log("query test2 balance:")
  // await queryAddressBalance(LCD_ENDPOINT, account2.address, "")

  console.log("claim reward:")
  let claimRewardRet = await executeContract(RPC_ENDPOINT, wallet, rewardAddress,
    {
      claim_rewards: {
        recipient: account.address,
      }
    }, "", parseCoins(""))
  console.log("claim reward ok!")

  console.log(`query ${account.address}`)
  await queryAddressBalance(LCD_ENDPOINT, account.address, "")

  console.log("query address test2 balance after claim reward:")
  await queryAddressBalance(LCD_ENDPOINT, account2.address, "")


  // let recAddress = "sei1tqm527sqmuw2tmmnlydge024ufwnvlv9e7draq";
  // // 2.2 send stable coin to test2 address
  // let sendCoinRet = await sendCoin(RPC_ENDPOINT, wallet, recAddress, "", coin(10000000, stable_coin_denom))
  // console.log(`send stable token to ${recAddress} succeed`);
  // console.log(`query ${recAddress} usdc balance:`);
  // let queryUsdcRet = await queryAddressBalance(LCD_ENDPOINT, recAddress, "")


  //***Test sending denom usdt token and sei coin**/
  // let receipientAddress = "sei1mlwyp04y5g95klqzq92tun0xsz7t5sef4h88a3";
  // // console.log("send denom usdt token to address:")
  // // await sendCoin(RPC_ENDPOINT, wallet, receipientAddress, "", coin(100000000, stable_coin_denom))
  // // console.log("send sei to address:")
  // // await sendCoin(RPC_ENDPOINT, wallet, receipientAddress, "", coin(100000000, "usei"))
  // console.log(`query ${receipientAddress} balance:`)
  // await queryAddressBalance(LCD_ENDPOINT, receipientAddress, "")

}

main().catch(console.log);

