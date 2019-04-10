create table users(
  id uuid primary key,
  email text not null unique,
  password text not null,
  avatar_url text not null,
  nickname text not null,
  is_verified boolean not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone not null
);
