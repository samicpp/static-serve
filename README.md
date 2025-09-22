# static-serve
static content webserver

## Features
- async runtime: allows for high demand whilst not needing large amount of recources
- all mime types: you can use almost all file extensions and expect the server to use the appropriate mime type
- directory handling: the server will look for a file starting with `index.*` or `[PARENT_DIRECTORY]*` effectively allowing any file type to be used as "index"

## Usage
`./static-runtime port directory`
example: `./static-runtime localhost:4096 ./static/`


## TODO
1. [x] tls support
2. [x] use HTTP/2 and allow H2C upgrade
3. [ ] use HTTP/3 (library currently doesnt support this)

