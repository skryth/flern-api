CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
	role TEXT NOT NULL CHECK (role IN ('admin', 'user'))
);

CREATE TABLE modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE lessons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id UUID NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL CHECK (task_type IN ('fill_code', 'multiple_choice', 'debug_code', 'string_cmp')),
    question TEXT NOT NULL,
    explanation TEXT NOT NULL
);

CREATE TABLE task_answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    answer_text TEXT NOT NULL,
	image TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE
);

-- User progress
CREATE TABLE user_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lesson_id UUID NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    status BOOLEAN NOT NULL DEFAULT FALSE,
	CONSTRAINT unique_user_lesson UNIQUE (user_id, lesson_id)
);

CREATE TABLE user_task_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    selected_answer_id UUID NOT NULL REFERENCES task_answers(id) ON DELETE CASCADE,
    is_correct BOOLEAN NOT NULL
);


--- Fake data, just for dev

-- Insert users
INSERT INTO users (username, password_hash, role)
VALUES 
  ('admin', '$argon2d$v=19$m=12,t=3,p=1$dXFsMjBjNWttZmswMDAwMA$UCzL5Yi9swPp9FCBzAlxzA', 'admin'),
  ('student1', '$argon2d$v=19$m=12,t=3,p=1$dXFsMjBjNWttZmswMDAwMA$UCzL5Yi9swPp9FCBzAlxzA', 'user');

-- Insert module
INSERT INTO modules (title, description, order_index)
VALUES 
  ('Rust Basics', 'Introduction to Rust programming language', 1);

-- Insert lessons
INSERT INTO lessons (module_id, title, content, order_index)
VALUES
  ((SELECT id FROM modules WHERE title = 'Rust Basics'), 'Variables and Mutability', 'Learn about let, mut, and constants in Rust.', 1),
  ((SELECT id FROM modules WHERE title = 'Rust Basics'), 'Ownership and Borrowing', 'Understand how Rust manages memory safely.', 2);

-- Insert tasks for lesson 1
INSERT INTO tasks (lesson_id, task_type, question, explanation)
VALUES
  ((SELECT id FROM lessons WHERE title = 'Variables and Mutability'), 'fill_code', 'Fill in the missing keyword: ___ x = 5;', 'You need to declare an immutable variable.'),
  ((SELECT id FROM lessons WHERE title = 'Variables and Mutability'), 'multiple_choice', 'Which keyword makes a variable mutable in Rust?', 'The mut keyword makes variables mutable.');

-- Answers for task1 (fill_code)
INSERT INTO task_answers (task_id, answer_text, image, is_correct)
VALUES
  ((SELECT id FROM tasks WHERE question LIKE 'Fill in the missing keyword%'), 'let', '', TRUE),
  ((SELECT id FROM tasks WHERE question LIKE 'Fill in the missing keyword%'), 'mut', '', FALSE),
  ((SELECT id FROM tasks WHERE question LIKE 'Fill in the missing keyword%'), 'const', '', FALSE);

-- Answers for task2 (multiple_choice)
INSERT INTO task_answers (task_id, answer_text, image, is_correct)
VALUES
  ((SELECT id FROM tasks WHERE question LIKE 'Which keyword makes a variable mutable%'), 'let', '', FALSE),
  ((SELECT id FROM tasks WHERE question LIKE 'Which keyword makes a variable mutable%'), 'mut', '', TRUE),
  ((SELECT id FROM tasks WHERE question LIKE 'Which keyword makes a variable mutable%'), 'var', '', FALSE);

-- Insert tasks for lesson 2
INSERT INTO tasks (lesson_id, task_type, question, explanation)
VALUES
  ((SELECT id FROM lessons WHERE title = 'Ownership and Borrowing'), 'debug_code', 'Fix the borrow checker error in the code snippet.', 'Remember: you cannot have mutable and immutable references at the same time.'),
  ((SELECT id FROM lessons WHERE title = 'Ownership and Borrowing'), 'string_cmp', 'What will be the result of comparing String::from(\"hi\") == \"hi\"?', 'Rust allows comparing String with &str because of the PartialEq implementation.');

-- Answers for task3 (debug_code)
INSERT INTO task_answers (task_id, answer_text, image, is_correct)
VALUES
  ((SELECT id FROM tasks WHERE question LIKE 'Fix the borrow checker%'), 'Remove the immutable reference.', '', TRUE),
  ((SELECT id FROM tasks WHERE question LIKE 'Fix the borrow checker%'), 'Use unsafe to bypass borrow checker.', '', FALSE);

-- Answers for task4 (string_cmp)
INSERT INTO task_answers (task_id, answer_text, image, is_correct)
VALUES
  ((SELECT id FROM tasks WHERE question LIKE 'What will be the result of comparing%'), 'true', '', TRUE),
  ((SELECT id FROM tasks WHERE question LIKE 'What will be the result of comparing%'), 'false', '', FALSE);

