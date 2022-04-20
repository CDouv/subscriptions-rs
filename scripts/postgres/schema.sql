CREATE SCHEMA subscriptions;

CREATE TABLE IF NOT EXISTS user_information (
    user_email VARCHAR(200) NOT NULL,
    user_entitlements jsonb,
    date_modified  date,
    PRIMARY KEY (user_email)
);



