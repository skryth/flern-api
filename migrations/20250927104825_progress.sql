-- Add migration script here
CREATE TABLE progress_tokens (
	id UUID PRIMARY KEY,
	token TEXT NOT NULL UNIQUE,
	user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	expires_at TIMESTAMPTZ NOT NULL,
	created_at TIMESTAMPTZ DEFAULT now()
)
