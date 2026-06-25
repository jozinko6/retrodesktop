# Metadata and media scraping

Planned provider boundary:

- ScreenScraper — primary metadata and media;
- SteamGridDB — grid, hero, logo, and icon artwork;
- IGDB — supplemental text metadata;
- LocalFilename — offline normalized-title fallback.

Matching must prefer platform, serial/internal ID, hashes, normalized filename, folder name, and fuzzy title evidence in that order. Region, revision, language, disc, track, and extension tags are removed before title matching.

Remote credentials are user supplied and must be stored in Windows Credential Manager. The current baseline does not include remote clients and therefore never asks for or stores credentials.

Default future media download: front cover, grid, hero, logo, and one screenshot. Video/manual are opt-in. Files live below `media/{systemId}/{gameId}` with relative SQLite paths, MIME/size limits, rate limiting, and orphan cleanup.
