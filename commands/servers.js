const DiscordMessageMenu = require ('./../menus');


module.exports.run = async (client, message, args, prefix) => {

    // populate local array
    let guilds = [];
    client.guilds.forEach(x => {
        guilds.push([x.name, x.members.size]);
    });

    // sort local array by count (greatest to least)
    guilds.sort((a, b) => {
        if (a[1] == b[1])
            return 0;

        return (a[1] > b[1])? -1 : 1;
    });

    // add menu items
    let items = [];
    guilds.forEach(x => {
        let count = x[1];
        let name = x[0];
        items.push('**' + name + '** - ' + count + ' users');
    });
    
    // display menu
    let menu = new DiscordMessageMenu(message, 'Discord Compiler Current Servers:', 0x00FF00, 15);
    menu.setNumbered(false);
    menu.buildMenu(items);
    menu.displayPage(0);
}

module.exports.help = {
    name:"servers",
    description:"displays all servers the bot is in",
    dev: true
}