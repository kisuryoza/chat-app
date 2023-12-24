CREATE TABLE IF NOT EXISTS accounts (
    user_id SERIAL PRIMARY KEY,
    login VARCHAR ( 50 ) UNIQUE NOT NULL,
    password VARCHAR ( 100 ) NOT NULL
);

