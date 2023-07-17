-- Add migration script here
CREATE TABLE rika_user (
    id SERIAL PRIMARY KEY NOT NULL,

    discord_id VARCHAR(255) UNIQUE,
    osu_id INT UNSIGNED UNIQUE
);

CREATE TABLE osu_user (
    id INT UNSIGNED PRIMARY KEY NOT NULL
);

CREATE TABLE osu_score (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL,
    osu_user_id INT UNSIGNED NOT NULL,

    mods INT UNSIGNED NOT NULL,
    map_id INT UNSIGNED NOT NULL,

    mode SMALLINT NOT NULL,
    
    FOREIGN KEY (osu_user_id) REFERENCES osu_user (id)
);


CREATE TABLE osu_performance (
    id BIGINT UNSIGNED PRIMARY KEY NOT NULL,

    aim FLOAT NOT NULL,
    speed FLOAT NOT NULL,
    accuracy FLOAT NOT NULL,
    flashlight FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    FOREIGN KEY (id) REFERENCES osu_score (id)
);
