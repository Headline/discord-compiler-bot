# Discord Compiler Bot
A simple open-source Discord bot which can compile code, display the output of the compiler, and run the program. This bot is great for prototyping ideas, or testing concepts on-the-fly with very little effort. It supports almost every programing language on the market (c++, c, java, python, ruby, javascript, go, php, lua, & more!). 

**Adding this bot to your server is as simple as clicking [here](https://discordapp.com/oauth2/authorize?client_id=504095380166803466&scope=bot&permissions=388160).**

**Just looking to try it out? Join our [support discord](https://discord.gg/ExraTaJ) and give it a shot!**

## Usage
For a tutorial about how to use this bot, feel free to view our [wiki](https://github.com/Headline/discord-compiler/wiki)!

## Hosting it yourself?
You will need to create a settings.json file alongisde index.js, the format is as follows:
```json
{
    "prefix": "YOUR_COMMAND_PREFIX_HERE",
    "token": "YOUR_TOKEN_HERE"
}
```
We also depend on the following npm packages.
- discord.js
- strip-ansi
- node-fetch

Once these are installed simply execute `node index.js`
