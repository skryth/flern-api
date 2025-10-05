
-- Lessons indexes to make the lookup faster
CREATE INDEX idx_lessons_module_id ON lessons(module_id);
CREATE INDEX idx_lessons_order_index ON lessons(order_index);
CREATE INDEX idx_lessons_module_order ON lessons(module_id, order_index);

-- Faster user progress lookup
CREATE INDEX idx_user_progress_user_lesson ON user_progress(user_id, lesson_id);
CREATE INDEX idx_user_progress_lesson_user ON user_progress(lesson_id, user_id);
CREATE INDEX idx_user_progress_user_status ON user_progress(user_id, status);

-- Modules
CREATE INDEX idx_modules_order_index ON modules(order_index);

-- Progress tokens
CREATE UNIQUE INDEX idx_progress_tokens_token ON progress_tokens(token);

-- Tasks
CREATE INDEX idx_tasks_lesson_id ON tasks(lesson_id);
