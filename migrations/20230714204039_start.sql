-- Add migration script here
CREATE TABLE rika_user (
    id SERIAL PRIMARY KEY,

    discord_id VARCHAR(255) UNIQUE,
    osu_id BIGINT UNIQUE
);

CREATE TABLE osu_user (
    id BIGINT PRIMARY KEY
);

CREATE TABLE osu_score (
    id BIGINT PRIMARY KEY,
    osu_user_id BIGINT NOT NULL,

    mode SMALLINT NOT NULL,
    
    FOREIGN KEY (osu_user_id) REFERENCES osu_user (id)
);


CREATE TABLE osu_performance (
    id BIGINT PRIMARY KEY,

    aim FLOAT NOT NULL,
    speed FLOAT NOT NULL,
    accuracy FLOAT NOT NULL,
    flashlight FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    FOREIGN KEY (id) REFERENCES osu_score (id)
);
