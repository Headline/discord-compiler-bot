// npm requirements
const Discord = require('discord.js');
const client = new Discord.Client({disableEveryone: true});
const fs = require('fs');

// source imports
const botconfig = require('./settings.json');
const WandBox = require ('./WandBox.js');
const Statistics = require('./statistics.js');

// discordbots.org
const dbllib = require('dblapi.js');

let dbl = null;
if (botconfig.dbltoken && botconfig.dbltoken.length > 0)
    dbl = new dbllib(botconfig.dbltoken, client);

const SupportServerModule = require('./supportserver.js');
let SupportServer = null;

// source import instantiations 
const servers = new Statistics.Servers(0, client, dbl);
const compilerAPI = new WandBox.Compilers(() => {
    console.log('compiler loading has completed!');
    compilerAPI.initialize();
});


// Add commands
console.log('loading commands...');
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

client.on('guildCreate', (g) => {
    servers.inc();
    SupportServer.postJoined(g);
    console.log(`joining ${g.name}`);
});
client.on('guildDelete', (g) => {
    servers.dec();
    SupportServer.postLeft(g);
    console.log(`leaving ${g.name}`);
});

// Callbacks
client.on('ready', () => {
    console.log('\'ready\' event executed. discord-compiler has started');

    servers.setCount(client.guilds.size);
    servers.updateAll();
    console.log(`existing in ${servers.getCount()} servers`);

    SupportServer = new SupportServerModule.SupportServer(client);
});

client.on('message', message => {
    if (!message.content.startsWith(botconfig.prefix)) return;
    if (message.author.bot) return;

    // strip !
    message.content = message.content.substring(botconfig.prefix.length);
    let args = message.content.split(" ").join('\n').split('\n');
    let commandfile = client.commands.get(args[0]);
    if (commandfile) {
        Statistics.Requests.DoRequest();

        if(commandfile.help.dev && message.author.id != botconfig.owner_id)
            return;

        commandfile.run(client, message, args, botconfig.prefix, compilerAPI, SupportServer);
    }
});

client.on('error', console.error);

// Pump them callbacks
client.login(botconfig.token);
