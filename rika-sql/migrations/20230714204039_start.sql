-- Add migration script here
CREATE TABLE rika_user (
    id SERIAL PRIMARY KEY NOT NULL,

    discord_id VARCHAR(255) UNIQUE,
    osu_id INT UNSIGNED
);

CREATE TABLE osu_user (
    id INT UNSIGNED PRIMARY KEY NOT NULL
);

CREATE TABLE osu_score (
    id BIGINT UNSIGNED NOT NULL,
    mode SMALLINT NOT NULL,

    osu_user_id INT UNSIGNED NOT NULL,

    mods INT UNSIGNED NOT NULL,
    map_id INT UNSIGNED NOT NULL,

    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
   
    PRIMARY KEY (id, mode),
    FOREIGN KEY (osu_user_id) REFERENCES osu_user (id) ON DELETE CASCADE
);


CREATE TABLE osu_performance (
    score_id BIGINT UNSIGNED PRIMARY KEY,

    aim FLOAT NOT NULL,
    speed FLOAT NOT NULL,
    accuracy FLOAT NOT NULL,
    flashlight FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    FOREIGN KEY (score_id) REFERENCES osu_score (id) ON DELETE CASCADE
);

CREATE TABLE taiko_performance (
    score_id BIGINT UNSIGNED PRIMARY KEY,

    accuracy FLOAT NOT NULL,
    difficulty FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    FOREIGN KEY (score_id) REFERENCES osu_score (id) ON DELETE CASCADE
);

CREATE TABLE mania_performance (
    score_id BIGINT UNSIGNED PRIMARY KEY,

    difficulty FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    FOREIGN KEY (score_id) REFERENCES osu_score (id) ON DELETE CASCADE
);
