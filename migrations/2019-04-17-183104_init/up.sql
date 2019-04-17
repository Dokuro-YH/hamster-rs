create table groups (
  id uuid primary key,
  display_name text not null,
  description text not null,
  created_at timestamp with time zone not null default now(),
  updated_at timestamp with time zone not null default now()
);

create type group_membership_type as enum ('user', 'group');

create table group_membership (
  group_id uuid not null references groups(id),
  member_id uuid not null,
  member_type group_membership_type not null default 'user',
  added timestamp with time zone not null default now(),
  primary key (group_id, member_id)
);
