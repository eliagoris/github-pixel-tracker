- Deploy with Fly.io (if you want)
- Call the pixel image on your GitHub profile project README using `![](your-fly-io.fly.io)`
- Profile views will be saved in the sqlite db.
- Query as needed using `fly ssh console` => `sqlite3 /mnt/data/access_log`
