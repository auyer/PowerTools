# community_settings_srv

Back-end for browsing community-contributed settings files for games.

### Technical

This does not use a database because I'm trying to speedrun the destruction of any semblance of performance for my filesystem. Everything is stored as a file, with symlinks to make it possible to find files multiple ways.
