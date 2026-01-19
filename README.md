# 👩‍💻 Discord Compiler
A Discord bot which can compile code, display the output of the compiler, and run the program. This bot is great for prototyping ideas, or testing concepts on-the-fly with very little effort. It supports almost every programing language you could name (c++, c, java, python, ruby, javascript, go, php, lua, & more!). 

## 🔗 Links
 - **[Invitation Link](https://discord.com/oauth2/authorize?client_id=504095380166803466&permissions=379968&scope=applications.commands%20bot)**
 
 - **[Support Discord](https://discord.gg/ExraTaJ)**
 
 - **[Donation Link](https://donatebot.io/checkout/505721414662225921) ❤️**

## 👩‍🏫 Usage
For a tutorial about how to use this bot, feel free to view our [wiki](https://github.com/Headline/discord-compiler-bot/wiki/1.-Getting-Started)!

## 🔰 Hosting it yourself?
### Docker
```yml
services:
  compiler-bot:
    image: ghcr.io/headline/discord-compiler-bot:latest
    restart: unless-stopped
    environment:
      # Required
      - BOT_TOKEN=
      - APPLICATION_ID=

      # Also required, but modifiable if you want
      - RUST_LOG=discord_compiler_bot
      - BOT_PREFIX=;
      - INVITE_LINK=https://discord.com/oauth2/authorize?client_id=504095380166803466&permissions=379968&scope=applications.commands%20bot
      - DISCORDBOTS_LINK=https://discordbots.org/bot/504095380166803466
      - GITHUB_LINK=https://github.com/Headline/discord-compiler
      - STATS_LINK=http://headlinedev.xyz/discord-compiler

      # Optional
      # - BOT_ID=
      # - SUCCESS_EMOJI_NAME=
      # - SUCCESS_EMOJI_ID=
      # - FAIL_EMOJI_NAME=
      # - FAIL_EMOJI_ID=
      # - LOADING_EMOJI_NAME=
      # - LOADING_EMOJI_ID=
      # - LOGO_EMOJI_NAME=
      # - LOGO_EMOJI_ID=
      # - COMPILE_LOG=
      # - JOIN_LOG=
      # - PANIC_LOG=
      # - VOTE_CHANNEL=
      # - DBL_TOKEN=
      # - DBL_WEBHOOK_PORT=
      # - DBL_WEBHOOK_PASSWORD=
```
### Manually
There's only two steps required to get this bot up-and-running. Our release builds only support 64-bit, if you'd like to run this on a different architecture you will have to compile the project yourself, this is also true if you wish to host this bot on MacOS.
1) Copy the repository's .env.example as a `.env` file & fill in required information
2) Download our [latest release](https://github.com/Headline/discord-compiler-bot/releases/) build & place it in the same directory as the `.env` file. For windows download `discord-compiler-bot.exe` & for linux download `discord-compiler-bot`.
3) Start the bot

## ⚖️ License
This project's license is the GNU AGPLv3 general purpose license. Review it [here](https://github.com/Headline/discord-compiler-bot/blob/master/LICENSE).

## 🖼️ Icons
Icons made by [Freepik](https://www.flaticon.com/authors/freepik) and [pixelmeetup](https://www.flaticon.com/authors/pixelmeetup) from [www.flaticon.com](https://www.flaticon.com/)
