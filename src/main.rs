use chrono::{DateTime,Utc};
use postgres::{Client, NoTls};
use dotenv::dotenv;
use serde_derive::Deserialize;
use serde_json::to_string_pretty;
use serde_json::json;
use serde_json::Value;
use serde_derive::Serialize;
use std::env;
use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION};
use reqwest::RequestBuilder;






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

//Three structs below to deal with incoming entitlement data
#[derive(Debug,Deserialize,Clone)]
struct SubscriptionEntitlementData {
    feature_id: String,
    feature_name: String,
    value: String,
    is_enabled:bool,
}
#[derive(Debug,Deserialize,Clone)]
struct SubscriptionEntitlement {
    subscription_entitlement:SubscriptionEntitlementData,
}
#[derive(Debug,Deserialize,Clone)]
struct SubscriptionEntitlements {
    list: Vec<SubscriptionEntitlement>
}

//Struct for desired entitlement format -> JSON
#[derive(Debug,Deserialize)]
struct EntitlementFormat {
    name:Option<String>,
    features:ValidatorFeatures<>,
}
#[derive(Debug,Deserialize)]
struct ValidatorFeatures {
    max_wells:Option<i64>,
    max_accounts:Option<i32>,
}

fn main() {

//connect to the db

let psql_pw:String = match dotenv::var("POSTGRES_PASSWORD"){
    Ok(val) => val,
    Err(E) => panic!("{:?}",E),
};
let mut db = Client::connect(format!("postgresql://postgres:{}@localhost:5438/postgres",psql_pw).as_str(),NoTls).unwrap();

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

    
#[tokio::main]
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
    
    println!("{:?}", resp.status());
    let resp_json = resp.json::<SubscriptionEntitlements>().await?;
    return Ok(resp_json)
    }
    

fn get_subscription_entitlements(subscriptions:HashMap<String,Vec<String>>) -> HashMap<String,SubscriptionData> {
    let mut subscription_all:HashMap<String,SubscriptionData> = HashMap::new();
    for (subscription_id,emails) in &subscriptions {
        let entitlement:Result<(SubscriptionEntitlements), Box<dyn std::error::Error>> = get_subscription_entitlement((subscription_id.to_string(),subscriptions[subscription_id].to_vec()));


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


fn clean_subscription_data(subscriptions:HashMap<String,SubscriptionData>) -> HashMap<String,Value> {
    //Hash Map representing (email,{feature_name:feature_value})
    // let mut clean_data: HashMap<String,HashMap<String,HashMap<String,String>>> = HashMap::new();
    let mut clean_data: HashMap<String,Value> = HashMap::new();
 
    for (subscription_id, subscription_data) in subscriptions {

        // transform entitlement to JSON:
        /*
        
        [
            {
                "name": "validator",
                "features": [
                    "max-wells": "10000",
                     "max-accounts": "2"
                ]
            }
        ]
        
        */
        



        //Generate Hashmap for <Product:Features>
        let mut product_entitlements:HashMap<String,HashMap<String,String>> = HashMap::new();

        //Grab list of entitlements from JSON data
        let entitlements = subscription_data.subscription_entitlement.list;

        for entitlement in entitlements {

            // Define product name
            let product_name = entitlement.subscription_entitlement.feature_id.split("--").collect::<Vec<&str>>()[0].to_string();
            //Define feature_name
            let feature_name:String = entitlement.subscription_entitlement.feature_id.to_string();
            //Define feature_value
            let feature_value:String = entitlement.subscription_entitlement.value.to_string();

            // Check if product name is already in larger HashMap (product_entitlements)
            if product_entitlements.contains_key(&product_name) {
                // If it's in product_entitlements, add features to existing product
                let mut features = product_entitlements[&product_name].clone();
    
                features.insert(feature_name,feature_value);
    
            
                product_entitlements.remove(&product_name);
                product_entitlements.insert(product_name,features.clone());
            } else {

                //If it's not in product_entitlements, create an empty HashMap and add features
                let mut features:HashMap<String,String> = HashMap::new();
                features.insert(feature_name,feature_value);
                product_entitlements.insert(product_name,features);
            }


        
        };
        let product_entitlements_json = json!(product_entitlements);
        
        
        //Loop through emails per subscriber, add key: email value: product_entitlements HashMap for every email
        for email in subscription_data.email {
            clean_data.insert(email.to_string(),product_entitlements_json.clone());
        }

    }

    
    return clean_data;
}

//Write a function to save formatted data (clean_data) into postgres


// Testing

//1 Get data from Chargebee
let data = get_data().unwrap();

//2 Pull subscription id's and email from Chargebee response
let subscription_info:HashMap<String,Vec<String>> = get_subscription_ids_emails(data);
println!("{:?}",subscription_info);

//3 Test single entitlement calculation

let single_subscriber = ("AzqYPESlckalh27S".to_string(),subscription_info["AzqYPESlckalh27S"].to_vec());
let single_entitlement:Result<(SubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber);
println!("{:?}", single_entitlement);


let single_subscriber = ("AzyuIISpk4F1U25R3".to_string(),subscription_info["AzyuIISpk4F1U25R3"].to_vec());
let single_entitlement:Result<(SubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber);
println!("{:?}", single_entitlement);

let single_subscriber = ("16BjoWSif9km010WB".to_string(),subscription_info["16BjoWSif9km010WB"].to_vec());
let single_entitlement:Result<(SubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber);
println!("{:?}", single_entitlement);

//3 Loop through subscription ids and get entitlements

let subscription_all:HashMap<String,SubscriptionData> = get_subscription_entitlements(subscription_info);
println!("SUBSCRIPTION_ALL {:?}",subscription_all);

// 4 Check Results
// println!("{:?}", subscription_all);


// println!("{:?}", subscription_all["AzqYPESlckalh27S"].subscription_id);
// println!("{:?}", subscription_all["AzqYPESlckalh27S"].email);
// println!("{:?}", subscription_all["AzqYPESlckalh27S"].subscription_entitlement);



//5 Create a HashMap of (email,entitlements)
let clean_data = clean_subscription_data(subscription_all);

println!("clean data");
println!("{:?}",clean_data);


//6 Populate postgres table with (email,entitlements)

// for (email, entitlements) in clean_data {
//     db.execute(
//         "INSERT INTO user_information (user_email,user_entitlements) VALUES (&1,&2)",
//         &[&email,&entitlements]).unwrap();
// };

}






    


