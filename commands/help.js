const Discord = require('discord.js');

module.exports.run = async (client, message, args) => {
    let output = "";
    let cmds = client.commands;
    cmds.forEach(element => {
        let name = element.help.name;
        let desc = element.help.description;
        output += [name + ' - ' + desc + '\n'];
    });

    message.channel.send(output);
}

module.exports.help = {
    name:"help",
    description:"displays all commands"
}