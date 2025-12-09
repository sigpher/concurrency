CREATE TABLE foodmate(
    id INTEGER PRIMARY KEY NOT NULL,
    item_id INTEGER not null,
    title text not null,
    status text not null,
    published_at text not null,
    effective_at text not null,
    issued_by text not null
);