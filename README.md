# scrobble-fix

My iPod had it's clock reset to 2001, and scrobbles have the incorrect date.

Parse the Rockbox scrobbler.log file, identify scrobbles with suspicious dates, and fix them.

---

AUDIOSCROBBLER/1.1 format is documented here:
- [Rockbox/rockbox - apps/plugins/lastfm_scrobbler.c](https://github.com/Rockbox/rockbox/blob/3c89adbdbdd036baf313786b0694632c8e7e2bb3/apps/plugins/lastfm_scrobbler.c#L29)
