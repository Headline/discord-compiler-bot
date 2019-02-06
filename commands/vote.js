const Discord = require('discord.js');
const botconfig = require('./../settings.json');

module.exports.run = async (client, message, args, prefix) => {
    const embed = new Discord.RichEmbed()
    .setTitle('Vote link:')
    .setColor(0xFF0000)
    .setDescription('Please vote using [this link]('+botconfig.discordbots_link+')!'
    + "\nThank you for voting!");
    message.channel.send(embed).catch();
};

module.exports.help = {
    name:"ping",
    description:"test command for the compiler bot",
    dev: false
}
