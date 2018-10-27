const Discord = require('discord.js');
const DiscordMessageMenu = require ('./../menus');

module.exports.run = async (client, message, args, prefix, compilerAPI) => {
    if (args.length < 2) {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`You must supply a language in order view it's supported compilers`)
        message.channel.send(embed).then((msg) => {
            let group = [message, msg];
            cmdlist.push(group);            
        });
        return;
    }
    let langs = compilerAPI.getCompilers(args[1].toLowerCase()); 
    if (langs === "None") {
        const embed = new Discord.RichEmbed()
        .setTitle('Error:')
        .setColor(0xFF0000)
        .setDescription(`The language *\'${args[1]}\'* is either not supported, or you have accidentially typed in the wrong language.` 
        + `Try using the *${prefix}languages* command to see supported languages!`);
        message.channel.send(embed).then((msg) => {
            let group = [message, msg];
            cmdlist.push(group);            
        });
        return;
    }
    let menu = new DiscordMessageMenu(message, `Supported \'${args[1].toLowerCase()}\' compilers:`, 0x00FF00, 15);
    menu.buildMenu(langs);
    menu.displayPage(0);
}

module.exports.help = {
    name:"compilers",
    description:"displays all compilers"
}