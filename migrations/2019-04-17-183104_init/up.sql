create table groups (
  id uuid primary key,
  display_name text not null unique,
  description text,
  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now()
);

create table group_membership (
  group_id uuid not null references groups(id),
  member_id uuid not null,
  member_type text not null default 'user',
  added timestamp with time zone not null default now(),
  primary key (group_id, member_id)
);

create table users (
  id uuid primary key,
  username text not null unique,
  password text not null,
  nickname text not null,
  avatar_url text not null,
  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now()
);
