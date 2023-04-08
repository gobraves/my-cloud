-- Add down migration script here
drop trigger if exists update_users_updated_at on users;
drop trigger if exists update_files_updated_at on files;
drop trigger if exists update_file_histories_updated_at on file_histories;
drop function update_modified_column() cascade;
