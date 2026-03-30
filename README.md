# Cangkang

A simple and minimal SSG, written in Rust with zero dependencies.

## how to deploy

just do this:

```bash
docker compose up --build -d
```

I added dockerfile and docker compose to make it easy to deploy & test.
i also added simple nginx conf as an example.

## how is this works

just place markdown file inside the `content/` then it will throw the html file to `dist/` directory. <br>
you can actually use this to test it or just use the docker compose build things in the top:

```bash
cargo run --release
```
