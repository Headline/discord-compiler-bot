const botconfig = require('./settings.json');

const Discord = require('discord.js');
const client = new Discord.Client();

client.on('ready', () => {
    console.log('I am ready!');
});

client.on('message', message => {
    if (!message.content.startsWith(botconfig.prefix)) return;
    if (message.author.bot) return;

    console.log('Message recieved: ' + message.content);

    // strip !
    message.content = message.content.substring(botconfig.prefix.length);
   
    // message fallthroughs
    if (message.content === 'ping') {
        message.channel.send('pong');
    }
});

client.login(botconfig.token);
