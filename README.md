The app is currently hosted on:
https://wednesday.mydud.es

To build the container:
```
docker build --tag immortaly007/wednesday .
```
To run it locally without Docker: `TZ=Europe/Amsterdam cargo run`

To run/test it:
```
docker run -d -p 8080:8080 -e TZ=Europe/Amsterdam immortaly007/wednesday
```

To publish
```
docker push immortaly007/wednesday
```
