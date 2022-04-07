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
struct ChargebeeList {
    list: Vec<ChargebeeSubscriptionInformation>,
}

#[derive(Debug,Deserialize)]
struct ChargebeeSubscriptionInformation {
    subscription: ChargebeeSubscription<>,

}


#[derive(Debug,Deserialize)]

struct ChargebeeSubscription {
    id: String,
    meta_data:Option<ChargebeeMetaData<>>,

}
#[derive(Debug,Deserialize,Clone)]
struct ChargebeeMetaData {
    accounts: Vec<String>,
}

#[derive(Debug,Deserialize,Clone)]
struct SubscriptionData {
    subscription_id: String,
    email: Vec<String>,
    subscription_entitlement: SubscriptionEntitlements<>,
}
#[derive(Debug,Deserialize,Clone)]
struct SubscriptionEntitlement {
    feature_id: String,
    feature_name: String,
    value: String,
    is_enabled:bool,
}
#[derive(Debug,Deserialize,Clone)]
struct SubscriptionEntitlements {
    list: Vec<SubscriptionEntitlement>
}

fn main() {


#[tokio::main]
async fn get_data() -> Result<(ChargebeeList), Box<dyn std::error::Error>> {

    dotenv().ok();

    let api_key = env::var("API_KEY")?;
    

  

    //Create request client
    let client = reqwest::Client::new();
    let resp = client
    .get("https://pod2-test.chargebee.com/api/v2/subscriptions")
    .basic_auth(api_key,Some(""))
    .send()
    .await?;


    let resp_json = resp.json::<ChargebeeList>().await?;



    return Ok(resp_json)
}



fn get_subscription_ids_emails(data:ChargebeeList) -> HashMap<String,Vec<String>> {

    let mut subscriptions:HashMap<String,Vec<String>> = HashMap::new();

    let subs = data.list;

    for (i,sub) in subs.iter().enumerate() {

        let no_email = ChargebeeMetaData {
            accounts: vec!["".to_string()]
        };
        
        let sub_id = &sub.subscription.id;
        let sub_email = sub.subscription.meta_data.as_ref().unwrap_or(&no_email);

        subscriptions.insert(sub_id.to_string(),sub_email.accounts.to_vec());
   
    };

    return subscriptions;

}

    

  





 


async fn get_subscription_entitlement(subscription:(String,Vec<String>)) -> Result<(SubscriptionEntitlements), Box<dyn std::error::Error>>{

    

    dotenv().ok();

    let api_key = env::var("API_KEY")?;
    
    //Create request client
    let client = reqwest::Client::new();
    let resp = client
    .get("https://pod2-test.chargebee.com/api/v2/subscriptions/".to_string()+ &subscription.0 +"/subscription_entitlements")
    .basic_auth(api_key,Some(""))
    .send()
    .await?;
    
    
    
    let resp_json = resp.json::<SubscriptionEntitlements>().await?;

    return Ok(resp_json)
    }
    
    
async fn get_subscription_entitlements(subscriptions:HashMap<String,Vec<String>>) -> HashMap<String,SubscriptionData> {
    let mut subscription_all:HashMap<String,SubscriptionData> = HashMap::new();
    for (subscription_id,emails) in &subscriptions {
        let entitlement:Result<(SubscriptionEntitlements), Box<dyn std::error::Error>> = get_subscription_entitlement((subscription_id.to_string(),subscriptions[subscription_id].to_vec())).await;


        //Create SubscriptionData object
        let subscription_data = SubscriptionData {
            //Populate subscription_id
            subscription_id: subscription_id.to_string(),
            //Populate email
            email: emails.to_vec(),
            //Populate entitlement
            subscription_entitlement:entitlement.unwrap(),
        };
        
        subscription_all.insert(subscription_id.to_string(),subscription_data);
        

        
    }

    return subscription_all;
}



//Write function to create HashMap of (email,entitlements)

// Testing

//1 Get data from Chargebee
let data = get_data().unwrap();

//2 Pull subscription id's and email from Chargebee response
let subscription_info:HashMap<String,Vec<String>> = get_subscription_ids_emails(data);
println!("{:?}",subscription_info);

//3 Test single entitlement calculation

let single_subscriber = ("16BjoWSif9km010WB".to_string(),subscription_info["16BjoWSif9km010WB"].to_vec());
let single_entitlement = get_subscription_entitlement(single_subscriber);
println!("{:?}", single_entitlement);
//3 Loop through subscription ids and get entitlements
// let subscription_all:HashMap<String,SubscriptionData> = get_subscription_entitlements(subscription_info);
//4 Check Results
// println!("{:?}", subscription_all);
//5 Create a HashMap of (email,entitlements)

//6 Populate postgres table with (email,entitlements)



}

    


