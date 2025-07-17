CREATE TABLE lottery_operator (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE game (
    id UUID PRIMARY KEY,
    lottery_operator_id INTEGER NOT NULL REFERENCES lottery_operator(id),
    name VARCHAR(255) NOT NULL
);
CREATE INDEX idx_game_lottery_operator_id ON game (lottery_operator_id);

CREATE TABLE draw (
    id SERIAL PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES game(id),
    status VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    open_time TIMESTAMPTZ NOT NULL,
    close_time TIMESTAMPTZ NOT NULL,
    draw_time TIMESTAMPTZ,
    winset_calculated_at TIMESTAMPTZ,
    winset_confirmed_at TIMESTAMPTZ
);
CREATE INDEX idx_draw_game_id ON draw (game_id);
CREATE INDEX idx_draw_status ON draw (status);
CREATE INDEX idx_draw_open_time ON draw (open_time);
CREATE INDEX idx_draw_close_time ON draw (close_time);

CREATE TABLE draw_level (
    id UUID PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES game(id),
    name VARCHAR(255) NOT NULL,
    number_of_selections INTEGER NOT NULL,
    min_value INTEGER NOT NULL,
    max_value INTEGER NOT NULL
);

CREATE TABLE wager (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    stake INTEGER NOT NULL,
    price INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_wager_user_id ON wager (user_id);

CREATE TABLE draw_wager (
    draw_id SERIAL NOT NULL REFERENCES draw(id),
    wager_id UUID NOT NULL REFERENCES wager(id)
);
CREATE INDEX idx_draw_wager_draw_id ON draw_wager (draw_id);
CREATE INDEX idx_draw_wager_wager_id ON draw_wager (wager_id);

CREATE TABLE board (
    id UUID PRIMARY KEY,
    wager_id UUID NOT NULL REFERENCES wager(id) ON DELETE CASCADE,
    game_type VARCHAR(255) NOT NULL DEFAULT 'NORMAL',
    system_game_level INTEGER
);
CREATE INDEX idx_board_wager_id ON board (wager_id);

CREATE TABLE selection (
    id UUID PRIMARY KEY,
    board_id UUID NOT NULL REFERENCES board(id) ON DELETE CASCADE,
    name VARCHAR(20) NOT NULL,
    values INTEGER[] NOT NULL
);
CREATE INDEX idx_selection_board_id ON selection (board_id);

CREATE TABLE win_class (
    id UUID PRIMARY KEY,
    game_id UUID NOT NULL REFERENCES game(id),
    name VARCHAR(255) NOT NULL,
    winclass_type VARCHAR(255) NOT NULL,
    factor INTEGER,
    constant BIGINT,
    percentage INTEGER,
    min_cap BIGINT,
    max_cap BIGINT
);
CREATE INDEX idx_win_class_game_id ON win_class (game_id);

CREATE TABLE win (
    id UUID PRIMARY KEY,
    wager_id UUID NOT NULL REFERENCES wager(id) ON DELETE CASCADE,
    win_class_id UUID NOT NULL REFERENCES win_class(id),
    amount REAL NOT NULL
);
CREATE INDEX idx_win_wager_id ON win (wager_id);
CREATE INDEX idx_win_win_class_id ON win (win_class_id);

CREATE TABLE audit_log (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(255) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_audit_log_entity_type ON audit_log (entity_type);
CREATE INDEX idx_audit_log_entity_id ON audit_log (entity_id);
CREATE INDEX idx_audit_log_created_at ON audit_log (created_at);