CREATE TABLE IF NOT EXISTS conversation_settings (
    id TEXT NOT NULL PRIMARY KEY,
    llm_model TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE INDEX idx_conversation_settings_conversation_id ON conversation_settings (conversation_id);
