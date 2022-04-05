use chrono::{DateTime,Utc};
use postgres::{Client, NoTls};
use dotenv::dotenv;
use serde_derive::Deserialize;
use std::env;
use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION};
use reqwest::RequestBuilder;





// //connect to the db
// let psql_pw = env::var("POSTGRES_PASSWORD")?;
// let mut db = Client::connect(format!("postgresql://postgres:{}@localhost:5438/postgres",psql_pw),NoTls).unwrap();

#[derive(Debug,Deserialize)]
struct List {
    list: Vec<SubscriptionInformation>,
}

#[derive(Debug,Deserialize)]
struct SubscriptionInformation {
    subscription: Subscription<>,

}


#[derive(Debug,Deserialize)]

struct Subscription {
    id: String,
    meta_data:Option<MetaData<>>,

}
#[derive(Debug,Deserialize,Clone)]
struct MetaData {
    accounts: Vec<String>,
}




fn main() {


#[tokio::main]
async fn get_data() -> Result<(List), Box<dyn std::error::Error>> {

    dotenv().ok();

    let api_key = env::var("API_KEY")?;
    println!("{}",api_key);

  

    //Create request client
    let client = reqwest::Client::new();
    let resp = client
    .get("https://pod2-test.chargebee.com/api/v2/subscriptions")
    .basic_auth(api_key,Some(""))
    .send()
    .await?;
    // println!("{:#?}", resp);

    let resp_json = resp.json::<List>().await?;


    // println!("{:?}", resp_json);
    // println!("{:?}", resp_json.list[0]);


    // Get subscription ids
    // for sub in resp_json.list.iter() {
    //     println!("{:?}",sub.subscription.id);
    //     println!("{:?}", sub.subscription.meta_data.as_ref().unwrap_or(MetaData));
    // }
    return Ok(resp_json)
}

fn get_subscription_ids(data:List) -> () {

    let mut subscriptions:Vec<String> =vec![];

    let subs = data.list;

    for (i,sub) in subs.iter().enumerate() {

        // let no_email = MetaData {
        //     accounts: vec!["".to_string()]
        // };
        
        let sub_id = &sub.subscription.id;

        subscriptions.push(sub_id.to_string());

        
        // println!("{:?}",&sub.subscription.meta_data.unwrap_or(no_email));
    };

    println!("{:?}",subscriptions);


}

let data = get_data();

println!("{:?}",get_subscription_ids(data.unwrap()));

}

    

    // Transform struct into HashMap
    
    // println!(":?}",data);

