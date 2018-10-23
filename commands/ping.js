const Discord = require('discord.js');

module.exports.run = async (client, message, args) => {
    message.channel.send('pong!');
    
}

module.exports.help = {
    name:"ping",
    description:"test command for the compiler bot"
}