CREATE SCHEMA subscriptions;

-- Create initital table
CREATE TABLE IF NOT EXISTS user_information (
    user_email VARCHAR(200) NOT NULL,
    user_entitlements jsonb,
    date_modified  TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (user_email)
);

--Add a couple of rows of dummy data to model as expired users
INSERT INTO user_information (user_email,user_entitlements,date_modified) 
               VALUES ('bob@bob.co','[{"name": "validator", "features": {"max-wells": "10000", "max-accounts": "2"}}]','2022-03-20 01:25:0');
              


