-- Add migration script here
CREATE TABLE rpoint_guildid (
        guildid VARCHAR,
        rpoint VARCHAR,
        UNIQUE (guildid, rpoint)
);

CREATE INDEX idx_rpoint_guildid ON rpoint_guildid USING BTREE (guildid, rpoint);
