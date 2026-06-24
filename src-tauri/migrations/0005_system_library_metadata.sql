ALTER TABLE games ADD COLUMN short_review TEXT;
ALTER TABLE games ADD COLUMN metadata_source TEXT;

INSERT OR IGNORE INTO systems(id, display_name) VALUES
('ps3','PlayStation 3'),('wiiu','Wii U'),('saturn','Sega Saturn'),
('dreamcast','Sega Dreamcast'),('segacd','Sega CD'),('sega32x','Sega 32X'),
('pcengine','PC Engine'),('neogeo','Neo Geo'),('arcade','Arcade'),
('scummvm','ScummVM'),('msx','MSX');
