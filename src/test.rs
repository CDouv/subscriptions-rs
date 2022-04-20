/* 
// Testing



//3 Test single entitlement calculation

// let single_subscriber = ("AzqYPESlckalh27S".to_string(),subscription_info["AzqYPESlckalh27S"].to_vec());
// let single_entitlement:Result<(ChargebeeSubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber).await;
// println!("{:?}", single_entitlement);


// let single_subscriber = ("AzyuIISpk4F1U25R3".to_string(),subscription_info["AzyuIISpk4F1U25R3"].to_vec());
// let single_entitlement:Result<(ChargebeeSubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber).await;
// println!("{:?}", single_entitlement);

// let single_subscriber = ("16BjoWSif9km010WB".to_string(),subscription_info["16BjoWSif9km010WB"].to_vec());
// let single_entitlement:Result<(ChargebeeSubscriptionEntitlements), Box<dyn std::error::Error>>= get_subscription_entitlement(single_subscriber).await;
// println!("{:?}", single_entitlement);

//3 Loop through subscription ids and get entitlements



// 4 Check Results
// println!("{:?}", subscription_all);


// println!("{:?}", subscription_all["AzqYPESlckalh27S"].subscription_id);
// println!("{:?}", subscription_all["AzqYPESlckalh27S"].email);
// println!("{:?}", subscription_all["AzqYPESlckalh27S"].subscription_entitlement);





println!("clean data");
for (email, entitlements) in &clean_data {
    println!("email: {:?}\n entitlements: {:?}\n\n",email,entitlements);
};


*/