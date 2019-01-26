const Discord = require('discord.js');

module.exports.run = async (client, message, args, prefix) => {
    const embed = new Discord.RichEmbed()
    .setTitle('Ping Output:')
    .setColor(0xFF0000)
    .setDescription('**Pong!**');
    message.channel.send(embed).catch();
};

module.exports.help = {
    name:"ping",
    description:"test command for the compiler bot",
    dev: false
}
