const Discord = require('discord.js');

module.exports.run = async (client, message, args, prefix) => {
    const embed = new Discord.RichEmbed()
    .setTitle('Ping Output:')
    .setColor(0xFF0000)
    .setDescription('**Pong!**');
    message.channel.send(embed).then((msg) => {
        let group = [message, msg];
        cmdlist.push(group);            
    });
}

module.exports.help = {
    name:"ping",
    description:"test command for the compiler bot"
}