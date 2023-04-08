-- Add up migration script here
-- postgresql 
create table file_histories (
    id bigserial primary key,
    fid bigint not null,
    file_version bigint not null,
    slices text[] not null,
    slices_hash text[] not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    foreign key (fid) references files(id),

    unique (fid, file_version)
);
