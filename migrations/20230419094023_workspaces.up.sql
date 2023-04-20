-- Add up migration script here
-- postgresql
create table workspaces (
   id uuid not null primary key,
   name varchar(255) not null,
   uid uuid not null,
   sync boolean not null default false,
   created_at timestamp not null default now(),
   updated_at timestamp not null default now()
);