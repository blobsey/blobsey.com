# Welcome

This is the source of my silly personal site. Enjoy. Or don't. I'm not your parent :)

## Development

*Why would you want to develop this? It's ***my*** site!!*

1. Download `wasm-pack`. This is a build tool of the `build.sh` script. Preferably put it somewhere in this project:
> ```
> wasm-pack build --target web --out-dir ../wasm/.tools
> ```

2. Put the domain name in a `.env` file at the root of the project. so something like 
```
  ├── cdk/
  ├── wasm/
  ├── website/
  ├── build.sh
    ...
  └── .env
```

Fill it with something like
```
DOMAIN_NAME="your-website.com"
```

3. Build and deploy!
```
./build.sh --deploy
```

## Local development
```
npx serve website -p 8000
```