CREATE TABLE IF NOT EXISTS conversations (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS messages (
    id TEXT NOT NULL PRIMARY KEY,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    conversation_id TEXT NOT NULL
);
CREATE INDEX idx_messages_conversation_id ON messages (conversation_id);
