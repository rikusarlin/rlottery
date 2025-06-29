CREATE TABLE lottery_operators (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE games (
    id UUID PRIMARY KEY,
    lottery_operator_id INTEGER NOT NULL REFERENCES lottery_operators(id),
    name VARCHAR(255) NOT NULL
);

CREATE TABLE draws (
    id SERIAL PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES games(id),
    status VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    open_time TIMESTAMPTZ NOT NULL,
    close_time TIMESTAMPTZ NOT NULL,
    draw_time TIMESTAMPTZ,
    winset_calculated_at TIMESTAMPTZ,
    winset_confirmed_at TIMESTAMPTZ
);

CREATE TABLE draw_levels (
    id UUID PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES games(id),
    name VARCHAR(255) NOT NULL,
    number_of_selections INTEGER NOT NULL,
    min_value INTEGER NOT NULL,
    max_value INTEGER NOT NULL
);

CREATE TABLE wagers (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    draw_id INTEGER NOT NULL REFERENCES draws(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE boards (
    id UUID PRIMARY KEY,
    wager_id UUID NOT NULL REFERENCES wagers(id)
);

CREATE TABLE selections (
    id UUID PRIMARY KEY,
    board_id UUID NOT NULL REFERENCES boards(id),
    draw_level_id UUID NOT NULL REFERENCES draw_levels(id),
    value INTEGER NOT NULL
);

CREATE TABLE win_classes (
    id UUID PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES games(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    factor REAL,
    constant REAL,
    percentage REAL,
    min_cap REAL,
    max_cap REAL
);

CREATE TABLE winnings (
    id UUID PRIMARY KEY,
    wager_id UUID NOT NULL REFERENCES wagers(id),
    win_class_id UUID NOT NULL REFERENCES win_classes(id),
    amount REAL NOT NULL
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(255) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
