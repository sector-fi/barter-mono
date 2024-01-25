use redis::{Commands, RedisError, Client};
use core::fmt;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use super::DexError;
use ethers::{
  contract::{abigen, ContractError}, 
  core::types::Address, 
  providers::{Provider, Http}, 
  utils::hex::FromHexError
};

pub struct TokenCache {
  client: Client,
}

abigen!(
  IERC20,
  r#"[
    function decimals() external view returns (uint8)
    function symbol() external view returns (string memory)
  ]"#,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
  pub addr: String,
  pub symbol: String,
  pub decimals: u8,
}
impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "Token {{ addr: {}, symbol: {}, decimals: {} }}", self.addr, self.symbol, self.decimals)
  }
}

lazy_static! {
  static ref REDIS_CACHE: Mutex<TokenCache> = Mutex::new(TokenCache {
      client: Client::open("redis://127.0.0.1/").expect("Failed to create Redis client"),
  });
}

impl From<RedisError> for DexError {
  fn from(err: RedisError) -> Self {
      DexError::Error(err.to_string())
  }
}

impl From<url::ParseError> for DexError {
  fn from(err: url::ParseError) -> Self {
      DexError::Error(err.to_string())
  }
}

impl From<FromHexError> for DexError {
  fn from(err: FromHexError) -> Self {
      DexError::Error(err.to_string())
  }
}

impl From<ContractError<Provider<Http>>> for DexError {
  fn from(err: ContractError<Provider<Http>>) -> Self {
      DexError::Error(err.to_string())
  }
}

impl TokenCache {
  pub fn instance() -> &'static Mutex<TokenCache> {
    &REDIS_CACHE
  }

  pub async fn get_token(&self, chain_id: &u64, address: &String) -> Result<Token, DexError> {
    let mut con = self.client.get_connection()?;
    let key = format!("{}:{}", chain_id, address);
    let result: Result<String, RedisError> = con.get(&key);
    match result {
      Ok(value) => {
        let token: Token = serde_json::from_str(&value)?;
        return Ok(token);
      },
      Err(e) => {
        // the token doesn't exist yet
        let token = self.get_token_from_chain(chain_id, address).await?;
        con.set(&key, serde_json::to_string(&token)?)?;
        return Ok(token)
      }
    }
  }

  pub async fn get_token_from_chain(&self, chain_id: &u64, address: &String) -> Result<Token, DexError> {
      println!("get_token_from_chain");
      // Connect to the network
      let provider = Provider::<Http>::try_from("https://rpc.ankr.com/eth")?;

      // Create an instance of the ERC20 contract
      let addr: Address = match address.parse() {
        Ok(addr) => addr,
        Err(e) => return Err(DexError::Error(e.to_string())),
      };

      let client = Arc::new(provider);
      let erc20 = IERC20::new(addr, client);

      // Call the decimals function
      let decimals = erc20.decimals().call().await?;
      println!("Decimals: {}", decimals);

      // Call the symbol function
      let symbol = erc20.symbol().call().await?;
      println!("Symbol: {}", symbol);
      let token = Token {
        addr: address.to_string(),
        symbol: symbol,
        decimals: decimals,
      };
      Ok(token)
  }
}