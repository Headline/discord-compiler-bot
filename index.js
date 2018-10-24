const botconfig = require('./settings.json');
const Discord = require('discord.js');
const WandBox = require ('./WandBox.js');

const client = new Discord.Client({disableEveryone: true});
const compilerAPI = new WandBox.Compilers(() => {
    console.log('compiler loading has completed!');
    compilerAPI.initialize();
});

// Add commands
console.log('loading commands...');
const fs = require('fs');
client.commands = new Discord.Collection();
fs.readdir('./commands/', (err, files) => {
    if (err)
        console.log(err);
    
    let jsfiles = files.filter(f => f.split('.').pop() === 'js');
    if (jsfiles.length == 0)
        return

    jsfiles.forEach((f, i) => {
        let props = require(`./commands/${f}`);
        console.log(`${f} command has been loaded!`);
        client.commands.set(props.help.name, props);
    });
});

// Callbacks
client.on('ready', () => {
    console.log('\'ready\' event executed. discord-compiler has started');
});

client.on('message', message => {
    if (!message.content.startsWith(botconfig.prefix)) return;
    if (message.author.bot) return;
    // strip !
    message.content = message.content.substring(botconfig.prefix.length);
    let args = message.content.split(" ");
    let commandfile = client.commands.get(args[0]);
    if (commandfile)
        commandfile.run(client, message, args, botconfig.prefix, compilerAPI)
});

// Pump them callbacks
client.login(botconfig.token);
