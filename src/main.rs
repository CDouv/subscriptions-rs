
use chrono::Local;
use postgres::{Client, NoTls};
use dotenv::dotenv;
use serde_derive::Deserialize;
use serde_json::json;
use serde_json::Value;
use serde_derive::Serialize;
use std::env;
use std::collections::HashMap;

// use reqwest::header::{HeaderMap, HeaderValue,AUTHORIZATION};


//Structs starting with 'Chargebee' are created to match incoming data from Chargebee API
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
    subscription_entitlement: ChargebeeSubscriptionEntitlements<>,
}

//Three structs below to deal with incoming entitlement data
#[derive(Debug,Deserialize,Clone)]
struct ChargebeeSubscriptionEntitlementData {
    feature_id: String,
    feature_name: String,
    value: String,
    is_enabled:bool,
}
#[derive(Debug,Deserialize,Clone)]
struct ChargebeeSubscriptionEntitlement {
    subscription_entitlement:ChargebeeSubscriptionEntitlementData,
}
#[derive(Debug,Deserialize,Clone)]
struct ChargebeeSubscriptionEntitlements {
    list: Vec<ChargebeeSubscriptionEntitlement>
}

//Struct for desired entitlement format -> JSON
#[derive(Debug,Serialize,Deserialize,Clone)]
struct ProductEntitlements {
    name:String,
    features:HashMap<String,String>,
}

//Function to connect to Chargebee API and return subscription data
fn get_data() -> Result<(ChargebeeList), reqwest::Error> {

    dotenv().ok();

    let api_key = env::var("API_KEY").unwrap();
    
    //Create request client
    let client = reqwest::blocking::Client::new();
    let resp = client
    .get("https://pod2-test.chargebee.com/api/v2/subscriptions")
    .basic_auth(api_key,Some(""))
    .send().unwrap();


    let resp_json = resp.json::<ChargebeeList>();



    return resp_json;
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



fn get_subscription_entitlement(subscription:(String,Vec<String>)) -> Result<(ChargebeeSubscriptionEntitlements), reqwest::Error>{

    dotenv().ok();

    let api_key = env::var("API_KEY").unwrap();
    
    //Create request client
    let client = reqwest::blocking::Client::new();
    let resp = client
    .get("https://pod2-test.chargebee.com/api/v2/subscriptions/".to_string()+ &subscription.0 +"/subscription_entitlements")
    .basic_auth(api_key,Some(""))
    .send()?;
    

    let resp_json = resp.json::<ChargebeeSubscriptionEntitlements>();
    return resp_json;
    }

//Function loops through subscription ids, pulls entitlement data using get_subscription_entitlement, and organizes data into subscription_id:SubscriptionData
fn combine_subscription_data(subscriptions:HashMap<String,Vec<String>>) -> HashMap<String,SubscriptionData> {
        let mut subscription_all:HashMap<String,SubscriptionData> = HashMap::new();
        for (subscription_id,emails) in &subscriptions {
            let entitlement:ChargebeeSubscriptionEntitlements = get_subscription_entitlement((subscription_id.to_string(),subscriptions[subscription_id].to_vec())).unwrap();
    
    
            //Create SubscriptionData object
            let subscription_data = SubscriptionData {
                //Populate subscription_id
                subscription_id: subscription_id.to_string(),
                //Populate email
                email: emails.to_vec(),
                //Populate entitlement
                subscription_entitlement:entitlement,
            };
            
            subscription_all.insert(subscription_id.to_string(),subscription_data);
            
    
            
        }
    
        return subscription_all;
    }



    
//Function takes raw email and entitlement data from Chargebee and converts to HashMap<email:String,entitlements:Vec<ProductEntitlements<>>

fn clean_subscription_data(subscriptions:HashMap<String,SubscriptionData>) -> HashMap<String,Value> {

    let mut email_product_subscriptions: HashMap<String,Value> = HashMap::new();
 
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
        let mut products_entitlements_map:HashMap<String,HashMap<String,String>> = HashMap::new();

        //Grab list of entitlements from JSON data
        let entitlements = subscription_data.subscription_entitlement.list;

        for entitlement in entitlements {

            // Define product name
            let product_name = entitlement.subscription_entitlement.feature_id.split("--").collect::<Vec<&str>>()[0].to_string();
            //Define feature_name
            let feature_name:String = entitlement.subscription_entitlement.feature_id.split("--").collect::<Vec<&str>>()[1].to_string();
            //Define feature_value
            let feature_value:String = entitlement.subscription_entitlement.value.to_string();

            // Check if product name is already in larger HashMap (products_entitlements_map)
            if products_entitlements_map.contains_key(&product_name) {
                // If it's in products_entitlements_map, add features to existing product
                let features = products_entitlements_map.get_mut(&product_name).unwrap();
    
                features.insert(feature_name,feature_value);
    
            
            } else {

                //If it's not in products_entitlements_map, create an empty HashMap and add features
                let mut features:HashMap<String,String> = HashMap::new();
                features.insert(feature_name,feature_value);
                products_entitlements_map.insert(product_name,features);
            }
        
        };

        //transform products_entitlements_map to ProductEntitlements struct
        let mut products_entitlements: Vec<ProductEntitlements> = vec!();
        

        for (product,features) in products_entitlements_map {
            let product_entitlements = ProductEntitlements {
                name: product.to_string(),
                features: features,
            };
            
            products_entitlements.push(product_entitlements);

        }
        
        let products_entitlements_json = json!(products_entitlements);
        
        //Loop through emails per subscriber, add key: email value: products_entitlements_map HashMap for every email
        for email in subscription_data.email {
            email_product_subscriptions.insert(email.to_string(),products_entitlements_json.clone());
        }

    }

  
   
    return email_product_subscriptions;
}



fn main() {

//connect to the db

let psql_pw:String = dotenv::var("POSTGRES_PASSWORD").unwrap();

let mut db = Client::connect(format!("postgresql://postgres:{}@localhost:5438/postgres",psql_pw).as_str(),NoTls).unwrap();

//Get data from Chargebee
let data = get_data().unwrap();

//Pull subscription id's and email from Chargebee response
let subscription_ids_emails:HashMap<String,Vec<String>> = get_subscription_ids_emails(data);

//Get subscription entitlements, organize data into subscription_id:SubscriptionData
let subscription_data:HashMap<String,SubscriptionData> = combine_subscription_data(subscription_ids_emails);

//Create a HashMap of (email,entitlements)
let clean_data = clean_subscription_data(subscription_data);

//Populate postgres table with (email,entitlements)

let timestamp = Local::now();

for (email, entitlements) in &clean_data {
    db.execute(
        "INSERT INTO user_information (user_email,user_entitlements,date_modified) 
               VALUES ($1,$2,$3) 
               ON CONFLICT (user_email)
               DO
                UPDATE SET user_entitlements = $2,
                           date_modified = $3"
                ,
        &[email,&entitlements,&timestamp]).unwrap();

    // Clear out information that was not part of upsert using date_modified timestamp as a reference
    db.execute("DELETE FROM user_information WHERE date_modified != $1", &[&timestamp]).unwrap();

       
};


}
 











    


