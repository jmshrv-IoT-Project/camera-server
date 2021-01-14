CREATE TABLE user_tokens (
    token uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL UNIQUE,
    user_id uuid NOT NULL,
    CONSTRAINT fk_user_id
        FOREIGN KEY (user_id)
            REFERENCES users (user_id)
            ON DELETE CASCADE
);