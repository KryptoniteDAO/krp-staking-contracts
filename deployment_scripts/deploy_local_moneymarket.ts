import {
  storeCode, instantiateContract, instantiateContract2, executeContract, queryStakingDelegations,
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

  // /**127.0.0.1 */
  let overseerCodeId = 273;
  let marketCodeId = 274;
  let custodybSeiCodeId = 275;
  let interestModelCodeId = 276;
  let distributionModelCodeId = 277;
  let oracleCodeId = 278;
  let aTokenCodeId = 279;
  let liquidationQueueCodeId = 280;

  let overseerAddress = "sei17pfs69uqtph8hh0f3hm0tpa48k2w3vvra8kn0q3uuq7r6wxhvpyqzx8vpc";
  let marketAddress = "sei1r6tua9uyx2desvz6peet3vvvprzm7rdgvxvxw3s53esy5az5rqysaj7uh7";
  let custodybSeiAddress = "sei1tedkfp6dh7uupcpyee68wvwf7zqntunvnwfjwrlskl20udteh7gqvcueg6";
  let aTokenAddress = "sei192ju375pg8h3whmt0ktw3sdr4tnzft0rc7d4uf0kehp2ypqdzsls3n2lc3";

  let interestModelAddress = "sei1qu8vqtcvaw0t8y78p77ss0895xlhunyhj9wgzg5vf3e7ew5s24cqn5yp4l";
  let distributionModelAddress = "sei1qu8vqtcvaw0t8y78p77ss0895xlhunyhj9wgzg5vf3e7ew5s24cqn5yp4l";
  let oracleAddress = "sei1fpjkzlgskptnsllmxdddaw4s29s3klsxtjh2ydhmktpentqhfcuqz8wfe5";
  let liquidationQueueAddress = "sei1dwz098jg2ykae3lg2w946u7dmq0p3e0tdj8urfzlqupdu72qsjqqrvn2q4";


  let hubAddress = "sei1nsugwwd4nsz0xny7g2y6rgmnuxghv6chcxjhzphn956l8tg7j8zsspg8z5";
  let rewardAddress = "sei1s83x9wmhwjw8venj3y4jamrap0mk42ngda5z2020dmqavvrfazys5wcale";
  let bSeiTokenAddress = "sei1c0a4h6c8qh6j0wmk998q590af9llth90d7dzlkk97naqzsdtepass3srvp";
  let rewardsDispatcherAddress = "sei1x7hl9rs2pcr23dtf6dncmjyj803wt2udusn2gkkc58tw27n2088swnvatm";
  let validatorsRegistryAddress = "sei1snccckkpjw2awl9gycpcsayl9ccesf48x8dqwarg5mzsskmkhz6spvghxw";
  let stSeiTokenAddress = "sei1hkchpay86jhdwtkjkth6pevly63kp8p347wmyf5nhcf9sk2rld9sv0gcav";

  //** test Release target wasm store code ,instantiate  money market flow **/
  // overseerCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_overseer.wasm")
  // marketCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_market.wasm")
  // custodybSeiCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_custody_bsei.wasm")
  // interestModelCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_interest_model.wasm")
  // distributionModelCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_distribution_model.wasm")
  // oracleCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_oracle.wasm")
  // aTokenCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../cw-plus/artifacts/cw20_base.wasm")
  // liquidationQueueCodeId = await storeCode(RPC_ENDPOINT, wallet, "../../krp-market-contracts/artifacts/moneymarket_liquidation_queue.wasm")


  console.log()

  const stable_coin_denom = "factory/sei1h3ukufh4lhacftdf6kyxzum4p86rcnel35v4jk/USDT";

  // //*instantiate contract
  // const [contract1, contract2] = await instantiateContract2(RPC_ENDPOINT, wallet, marketCodeId,
  // {
  //   anc_emission_rate: "6793787.950524103374549206", 
  //   atoken_code_id: aTokenCodeId,
  //   max_borrow_factor: "0.95",
  //   owner_addr: account.address,
  //   reserve_factor: "0.0",
  //   stable_denom: stable_coin_denom
  // }, coins(1000000, stable_coin_denom), "money market contract")
  // marketAddress = contract1;
  // aTokenAddress = contract2;

  // interestModelAddress = await instantiateContract(RPC_ENDPOINT, wallet, interestModelCodeId,
  //     {
  //       base_rate: "0.000000004076272770",
  //       interest_multiplier: "0.000000085601728176",
  //       owner: account.address
  //     }, parseCoins(""), "interest model contract")


  // distributionModelAddress = await instantiateContract(RPC_ENDPOINT, wallet, distributionModelCodeId,
  //     {
  //       decrement_multiplier: "0.997102083349256160", 
  //       emission_cap: "20381363.851572310123647620",
  //       emission_floor: "6793787.950524103374549206", 
  //       increment_multiplier: "1.007266723782294841",
  //       owner: account.address
  //     }, parseCoins(""), "distribution model contract")

  // oracleAddress = await instantiateContract(RPC_ENDPOINT, wallet, oracleCodeId,
  //     {
  //       base_asset: stable_coin_denom,
  //       owner: account.address
  //     }, parseCoins(""), "oracle contract")


  // liquidationQueueAddress = await instantiateContract(RPC_ENDPOINT, wallet, liquidationQueueCodeId,
  //   { 
  //     owner: account.address,
  //     oracle_contract: oracleAddress,
  //     stable_denom: stable_coin_denom,
  //     safe_ratio: "0.8",
  //     bid_fee: "0.01",
  //     liquidator_fee: "0.01",
  //     liquidation_threshold: "500",
  //     price_timeframe: 86400,
  //     waiting_period: 600,
  //     overseer: overseerAddress
  //   }, parseCoins(""), "liquidation queue contract")


  // overseerAddress = await instantiateContract(RPC_ENDPOINT, wallet, overseerCodeId,
  //     {
  //       anc_purchase_factor: "0.1",
  //       buffer_distribution_factor: "0.1",
  //       collector_contract: "sei1xxrlcs6kekmh63ks26yuf47qxdrkkqw0srvh7w", //ANC里面一个合约协议收入10%(暂时用一个临时地址代替)
  //       epoch_period: 1681,
  //       liquidation_contract: liquidationQueueAddress,
  //       market_contract: marketAddress,
  //       oracle_contract: oracleAddress,
  //       owner_addr: account.address,
  //       price_timeframe: 86400,
  //       stable_denom: stable_coin_denom,
  //       target_deposit_rate: "0.000000040762727704",
  //       threshold_deposit_rate: "0.000000030572045778",
  //       dyn_rate_epoch: 8600,
  //       dyn_rate_maxchange: "0.005",
  //       dyn_rate_yr_increase_expectation: "0.001",
  //       dyn_rate_min: "0.000001",
  //       dyn_rate_max: "0.0000012"
  //     }, parseCoins(""), "overseer contract")


  // custodybSeiAddress = await instantiateContract(RPC_ENDPOINT, wallet, custodybSeiCodeId,
  //     {
  //     basset_info: {
  //       decimals: 6,
  //       name: "Bonded Sei",
  //       symbol: "BSEI"
  //     },
  //     collateral_token: bSeiTokenAddress, 
  //     liquidation_contract: liquidationQueueAddress,
  //     market_contract: marketAddress,
  //     overseer_contract: overseerAddress,
  //     owner: account.address,
  //     reward_contract: rewardAddress,  
  //     stable_denom: stable_coin_denom
  //     }, parseCoins(""),"custody bond sei contract")

  //   console.log()

  //   console.log(`OVERSEER_CONTRACT = ${overseerAddress}`)
  //   console.log(`MONEY_MARKET_CONTRACT = ${marketAddress}`)
  //   console.log(`CUSTODY_BSEI_CONTRACT = ${custodybSeiAddress}`)
  //   console.log(`A_TOKEN_ADDRESS = ${aTokenAddress}`)
  //   console.log(`INTEREST_MODEL_CONTRACT = ${interestModelAddress}`)
  //   console.log(`DISTRIBUTION_MODEL_CONTRACT = ${distributionModelAddress}`)
  //   console.log(`ORACLE_CONTRACT = ${oracleAddress}`)
  //   console.log(`LIQUIDATION_QUEUE_CONTRACT = ${liquidationQueueAddress}`)

  ///////////////////////configure/////////////////////////////////////////////////
  /////////////////////////////////////////////////////////////////////////////////
  // console.log("Update overseer's config ...")
  //   let updateOverseerCfg = await executeContract(RPC_ENDPOINT, wallet, overseerAddress, 
  //     {
  //       update_config : 
  //         {     
  //           "liquidation_contract": liquidationQueueAddress, 
  //           "epoch_period": 90, 
  //         }
  //     }, "",  parseCoins(""))
  //   console.log("Update overseer's config end")

  //   console.log("Update custodybSei's config...")
  //   let updateCustodybSeiCfg = await executeContract(RPC_ENDPOINT, wallet, custodybSeiAddress, 
  //     {
  //       update_config: {
  //         owner: account.address, 
  //         liquidation_contract: liquidationQueueAddress, 
  //       }
  //     }, "", parseCoins(""))
  //   console.log("Update custodybSei's config end")

  //   console.log("Update liquidation_queue_contract config...")
  //   let updateLiquidationCfgRet = await executeContract(RPC_ENDPOINT, wallet, liquidationQueueAddress, 
  //     {
  //       update_config: {
  //         owner: account.address, 
  //         oracle_contract: oracleAddress, 
  //         safe_ratio: "0.8", 
  //         bid_fee: "0.01", 
  //         liquidator_fee: "0.01",
  //         liquidation_threshold: "500", 
  //         price_timeframe: 86400,
  //         waiting_period: 600,
  //         overseer: overseerAddress,
  //       }
  //     }, "", parseCoins(""))
  //   console.log("Update liquidation_queue_contract config end")

  //   //config overseer contract add collateral token
  //   console.log("Updating overseer's add collateral whitelist...")
  //   await executeContract(RPC_ENDPOINT, wallet, overseerAddress, 
  //     {
  //       whitelist : 
  //         {
  //           name: "Bond Sei",
  //           symbol: "bSEI",
  //           collateral_token: bSeiTokenAddress,
  //           custody_contract: custodybSeiAddress,
  //           max_ltv: "0.65"
  //         }
  //     }, "", parseCoins(""))
  //   console.log("Updating overseer's add collateral whitelist end")


  //   console.log("register contracts for money market...")
  //   await executeContract(RPC_ENDPOINT, wallet, marketAddress, 
  //     {
  //       register_contracts: {
  //         overseer_contract: overseerAddress,
  //         interest_model: interestModelAddress,
  //         distribution_model: distributionModelAddress,
  //         collector_contract: bSeiTokenAddress,
  //         distributor_contract: rewardsDispatcherAddress,
  //       } 
  //     }, "", parseCoins(""))
  //     console.log("register contracts for money market end")

  // console.log("add whitelist collateral to liquidation_queue_contract...")
  // let whitelistCollateralRet = await executeContract(RPC_ENDPOINT, wallet, liquidationQueueAddress,
  //   {
  //     whitelist_collateral: {
  //       collateral_token: bSeiTokenAddress, 
  //       bid_threshold: "500000000", 
  //       max_slot: 30,
  //       premium_rate_per_slot: "0.01",
  //     }
  //   }, "", parseCoins(""));
  // console.log("add whitelist collateral to liquidation_queue_contract end")
  ////////////////////////////////////////////////////////////////////////////////////////////////////
  /////////////////////////           Initialization End            //////////////////////////////////
  ////////////////////////////////////////////////////////////////////////////////////////////////////








  ////////////////////////////////////////test main process//////////////////////////////////////////////
  ///////////////////////////////////////===================//////////////////////////////////////////////


  ///2. deposit_stable test 
  //2.1 deposit stable to money market
  // let depositRet = await executeContract(RPC_ENDPOINT, wallet, marketAddress, { "deposit_stable": {} }, "", coins(1000000000, stable_coin_denom))
  // // console.log(`${JSON.stringify(depositRet)}`);
  // console.log("deposit stable token ok");

  // // 2.2 send stable coin to other address
  let receviceAddress = account2.address;
  await sendCoin(RPC_ENDPOINT, wallet, receviceAddress, "", coin(1000, stable_coin_denom))
  await sendCoin(RPC_ENDPOINT, wallet, receviceAddress, "", coin(100, "usei"))
  console.log("send stable token to test2 succeed");

  ///3. Deposits collateral. 
  /// Issued when a user sends bAsset tokens to the Custody contract.
  let depositCollateral = await executeContract(RPC_ENDPOINT, wallet, bSeiTokenAddress,
    {
      send: {
        contract: custodybSeiAddress, amount: "1000000",
        msg: Buffer.from(JSON.stringify({ "deposit_collateral": {} })).toString('base64')
      }
    }, "", parseCoins(""))
  console.log("deposit collateral ok");

  ///4. Lock Collateral 
  let lockCollateralRet = await executeContract(RPC_ENDPOINT, wallet, overseerAddress,
    {
      lock_collateral: {
        collaterals: [
          [bSeiTokenAddress, "1000000"]
        ]
      }
    }, "", parseCoins(""))
  console.log("lock collateral ok");


  // // // // 5. register feeder for asset in oracle contract
  let registerFeederRet = await executeContract(RPC_ENDPOINT, wallet, oracleAddress,
    {
      register_feeder: {
        asset: bSeiTokenAddress,
        //feeder: "sei1xm3mccak0yjfts96jszdldxka6xkw00ywv6au0",
        feeder: account.address,
      }
    }, "", parseCoins(""))
  console.log("register feeder ok");


  // // // //  5.1 feed Price
  // // // //  Feeds new price data. Can only be issued by the owner.    
  await executeContract(RPC_ENDPOINT, wallet, oracleAddress,
    {
      feed_price: {
        prices: [
          [bSeiTokenAddress, "100"]
        ]
      }
    }, "", parseCoins(""))
  console.log("feeder price ok");


  //6. borrow stable
  //Borrows stablecoins from Anchor.
  let borrowStableRet = await executeContract(RPC_ENDPOINT, wallet, marketAddress,
    {
      borrow_stable: {
        borrow_amount: "10000000",
        to: account.address
      }
    }, "", parseCoins(""));
  console.log("borrow stable ok");


  ///7. query borrow stable coin info
  console.log("query borrow stable coin info:");
  await queryWasmContract(RPC_ENDPOINT, wallet, marketAddress,
    {
      borrower_info: {
        borrower: account.address,
      }
    });

  console.log("query market state:")


  ////////////////////test////////////////////////////////////////////////////////////////////////////////
  ///////////////liquidatequeue///////////////////////////////////////////////////////////////////////////////
  ////14.2 feed Price
  //Feeds new price data. Can only be issued by the owner.    
  await executeContract(RPC_ENDPOINT, wallet, oracleAddress,
    {
      feed_price: {
        prices: [
          [bSeiTokenAddress, "50"]
        ]
      }
    }, "", parseCoins(""))
  console.log("feeder price ok");

  // //15.Liquidate Collateral
  // //15.2 query collateral borrow limit
  console.log("query collateral borrow limit:");
  let curretTimestamp = Date.parse(new Date().toString()) / 1000;
  await queryWasmContract(RPC_ENDPOINT, wallet, overseerAddress,
    {
      borrow_limit: {
        borrower: account.address,
        block_time: curretTimestamp,
      }
    })


  //  15.3 liquidate collateral 
  // Submits a new bid for the specified Cw20 collateral with the specified premium rate. 
  // Requires stablecoins to be sent beforehand.
  let submitBidRet = await executeContract(RPC_ENDPOINT, wallet2, liquidationQueueAddress,
    {
      submit_bid: {
        collateral_token: bSeiTokenAddress,
        premium_slot: 10
      }
    }, "", coins(100000000, stable_coin_denom))
  console.log("submit bid ok");
  ///////////////////////////////////////////////////////////////////////////////
  //                                                                           //
  ///////////////////////////////////////////////////////////////////////////////

  //Gets the collateral balance of the specified borrower.
  console.log(`query collateral balance:`);
  await queryWasmContract(RPC_ENDPOINT, wallet, custodybSeiAddress,
    {
      borrower: {
        address: account.address
      }
    })
  //console.log(`${JSON.stringify(queryCollateralBalance)}`);
  //Feeds new price data. Can only be issued by the owner.    
  await executeContract(RPC_ENDPOINT, wallet, oracleAddress,
    {
      feed_price: {
        prices: [
          [bSeiTokenAddress, "5"]
        ]
      }
    }, "", parseCoins(""))
  console.log("feeder price ok");


  // ///12.1 query borrow stable coin info 
  console.log("query borrow stable coin info:");
  await queryWasmContract(RPC_ENDPOINT, wallet, marketAddress,
    {
      borrower_info: {
        borrower: account.address,
        block_height: null
      }
    });


  //15.2 query collateral borrow limit
  console.log("query collateral borrow limit:");
  curretTimestamp = Date.parse(new Date().toString()) / 1000;
  await queryWasmContract(RPC_ENDPOINT, wallet, overseerAddress,
    {
      borrow_limit: {
        borrower: account.address,
        block_time: curretTimestamp,
      }
    })


  console.log("query overseer white list:");
  await queryWasmContract(RPC_ENDPOINT, wallet, overseerAddress,
    {
      whitelist: {
        collateral_token: bSeiTokenAddress,
        start_after: null,
        limit: null
      }
    })
  
  /////////////////////////////////////////////////////////////////////
  ////must execute submit bid operation to avoid error ////////////////
  console.log("query liquidation amount:");
  await queryWasmContract(RPC_ENDPOINT, wallet, liquidationQueueAddress,
    {
      liquidation_amount: {
        borrow_amount: "10000067",
        borrow_limit: "6500000",
        collaterals: [
          [bSeiTokenAddress, "2000000"]
        ],
        collateral_prices: [
          "5"
        ]
      }
    })


  console.log("query liquidatequeue config:")
  await queryWasmContract(RPC_ENDPOINT, wallet, liquidationQueueAddress,
    {
      config: {}
    })

  //15.4 liquidate collateral call contract custodbSei =====step4==============
  let liquidateRet = await executeContract(RPC_ENDPOINT, wallet2, overseerAddress,
    {
      liquidate_collateral: {
        borrower: account.address,
      }
    }, "", parseCoins(""))

  //15.5 query liquidatequeue config
  await queryWasmContract(RPC_ENDPOINT, wallet, liquidationQueueAddress, { config: {} });


  /// 15.6 query liquidate pool
  await queryWasmContract(RPC_ENDPOINT, wallet, liquidationQueueAddress,
    {
      bid_pool: {
        collateral_token: bSeiTokenAddress,
        bid_slot: 10,
      }
    });

  console.log("query reward state:");
  await queryWasmContract(RPC_ENDPOINT, wallet, rewardAddress, { state: {} })

  console.log("claim rewards:");
  let claimRewarsRet = await executeContract(RPC_ENDPOINT, wallet, hubAddress, { update_global_index: {} }, "", parseCoins(""))


  // console.log("query accured reward:")
  await queryWasmContract(RPC_ENDPOINT, wallet, rewardAddress,
    {
      accrued_rewards:
      {
        address: account.address
      }
    })


  await queryWasmContract(RPC_ENDPOINT, wallet, interestModelAddress, { "config": {} });


  ///////////////////////////////////////////////////////////////////////////////////////////////////////////
  await queryWasmContract(RPC_ENDPOINT, wallet, marketAddress,
    {
      state: {
        //   block_height : 6371152
      }
    })


}

main().catch(console.log);

function deposit_stable(this: any, key: string, value: any) {
  throw new Error('Function not implemented.');
}

