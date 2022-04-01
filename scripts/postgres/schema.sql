CREATE SCHEMA subscriptions;

CREATE TABLE IF NOT EXISTS user_information (
    user_id varchar(200) NOT NULL,
    PRIMARY KEY (user_id)
);



{"products":[{"features":[{"name":"max-accounts","value":2}],"name":"validator"}]}