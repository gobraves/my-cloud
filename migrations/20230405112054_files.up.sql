-- Add up migration script here
-- postgresql 
create table files (
    id bigint not null primary key,
    uid uuid not null,
    ws_id uuid not null,
    filename varchar(255) not null,
    parent_dir_id bigint not null,
    is_deleted boolean not null default false,
    size bigint not null,
    is_dir boolean not null,
    version bigint not null default 0,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),

    unique (uid, filename, parent_dir_id)
);

create index file_uid_idx on files (uid);
create index uid_parent_dir_id_idx on files (uid, parent_dir_id);
