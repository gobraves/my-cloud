-- Add up migration script here
-- postgresql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table users (
    id uuid primary key,
    name varchar(255) not null,
    email varchar(255) not null,
    password_hash varchar(255) not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),

    unique (email)
);
