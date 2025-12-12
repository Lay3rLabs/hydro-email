Sometimes it's helpful to debug against a local mail server instead of using real credentials on Google etc.

## Using Greenmail

Start the local mail server

```bash
task backend:mail-server-start
```

Send an email (then check again!)

```bash
task backend:mail-server-send
```

Now, assuming you've set the `.env` credentials for a local test user, you can check the inbox as usual:

```bash
task components:exec-read-mail
```

When you're finished, you can stop the local mail server:

```bash
task backend:mail-server-stop
```
