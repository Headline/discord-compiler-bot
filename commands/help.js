const DiscordMessageMenu = require ('./../menus');

module.exports.run = async (client, message, args, prefix) => {
    let items = [];
    client.commands.forEach(element => {
        let name = element.help.name;
        let desc = element.help.description;
        let dev = element.help.dev;

        if (!dev)
            items.push('**' + prefix + name + '** - ' + desc + '\n');
    });

    let menu = new DiscordMessageMenu(message, 'Discord Compiler Bot Help Menu:', 0x00FF00, 6);
    menu.setNumbered(false);
    menu.buildMenu(items);
    menu.displayPage(0);
}

module.exports.help = {
    name:"help",
    description:"displays all commands",
    dev: false
}