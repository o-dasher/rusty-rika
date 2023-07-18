-- Add migration script here
ALTER TABLE osu_score
ADD COLUMN created_at TIMESTAMP DEFAULT NOW();

