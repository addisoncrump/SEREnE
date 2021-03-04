# SEREnE

![serene swan swimming with mr krabs](serene.png)

Sandboxed Environment for Reverse Engineering 'n' Education!

## Ok, what is it?

As it says on the tin: a bot which generates sandboxes for you to perform basic reverse engineering tasks in
collaboratively. I built this mostly for a reverse engineering class that I'm taking this semester so that we could use
a shared sandbox preloaded with the tools we needed for the class.

## What can it do?

It has quite a few packages pre-installed, and all you need to do to modify that is add to [the sandbox
Dockerfile](sandbox/Dockerfile).

## How do I use it?

1. Acquire a public-facing (or VPN-accessible or ZeroTier-accessible) Linux server, and log in.
2. Install [Docker](https://docs.docker.com/get-docker/).
3. `docker build -t serene .` in the repository folder and `docker build -t serene-sandbox .` in the [sandbox
   folder](sandbox).
4. [Make a bot!](https://discord.com/developers/applications)
5. Write an appropriate configuration file with the following content:
```toml
token = "YOUR_BOT_TOKEN"
host = "your.host.example.com"
owner = 1234567890 # your user id
```
6. `docker run -d --restart=always -v /var/run/docker.sock:/var/run/docker.sock -v $(pwd)/serene.toml:/serene.toml serene`
7. Profit.

Use `~help` to get started!

## Your bot does very little!

As of right now, it's admittedly not the most configurable bot in the world. More of a proof-of-concept than anything
professional.

## I was able to escape the sandbox!

Awesome! Tell me how and I'll give you brownie points. Tell Docker and they'll probably give you cash.

## I had a probl-

Please visit the [issues](https://github.com/VTCAKAVSMoACE/SEREnE/issues) page to report a bug.