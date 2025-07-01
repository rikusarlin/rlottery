-- Add index to games.lottery_operator_id
CREATE INDEX idx_games_lottery_operator_id ON games (lottery_operator_id);

-- Add indexes to draws table
CREATE INDEX idx_draws_game_id ON draws (game_id);
CREATE INDEX idx_draws_status ON draws (status);
CREATE INDEX idx_draws_open_time ON draws (open_time);
CREATE INDEX idx_draws_close_time ON draws (close_time);

-- Add indexes to wagers table
CREATE INDEX idx_wagers_user_id ON wagers (user_id);
CREATE INDEX idx_wagers_draw_id ON wagers (draw_id);

-- Add index to boards table
CREATE INDEX idx_boards_wager_id ON boards (wager_id);

-- Add indexes to selections table
CREATE INDEX idx_selections_board_id ON selections (board_id);
CREATE INDEX idx_selections_draw_level_id ON selections (draw_level_id);

-- Add index to win_classes table
CREATE INDEX idx_win_classes_game_id ON win_classes (game_id);

-- Add indexes to winnings table
CREATE INDEX idx_winnings_wager_id ON winnings (wager_id);
CREATE INDEX idx_winnings_win_class_id ON winnings (win_class_id);

-- Add indexes to audit_logs table
CREATE INDEX idx_audit_logs_entity_type ON audit_logs (entity_type);
CREATE INDEX idx_audit_logs_entity_id ON audit_logs (entity_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs (created_at);
